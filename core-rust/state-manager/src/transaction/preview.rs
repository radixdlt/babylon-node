use node_common::locks::RwLock;
use radix_engine::transaction::{PreviewError, TransactionReceipt, TransactionResult};
use radix_engine_store_interface::db_key_mapper::SpreadPrefixKeyMapper;
use std::ops::{Deref, Range};
use std::sync::Arc;

use crate::query::{StateManagerSubstateQueries, TransactionIdentifierLoader};
use crate::staging::ReadableStore;
use crate::store::traits::QueryableProofStore;
use crate::transaction::*;
use crate::{PreviewRequest, ProcessedCommitResult, SubstateChange};
use radix_engine_common::prelude::*;
use transaction::model::*;
use transaction::signing::secp256k1::Secp256k1PrivateKey;
use transaction::validation::NotarizedTransactionValidator;
use transaction::validation::ValidationConfig;

/// A transaction preview runner.
pub struct TransactionPreviewer<S> {
    store: Arc<RwLock<S>>,
    execution_configurator: Arc<ExecutionConfigurator>,
    validation_config: ValidationConfig,
}

pub struct ProcessedPreviewResult {
    pub receipt: TransactionReceipt,
    pub substate_changes: Vec<SubstateChange>,
}

impl<S> TransactionPreviewer<S> {
    pub fn new(
        network: &NetworkDefinition,
        store: Arc<RwLock<S>>,
        execution_configurator: Arc<ExecutionConfigurator>,
    ) -> Self {
        Self {
            store,
            execution_configurator,
            validation_config: ValidationConfig::default(network.id),
        }
    }
}

impl<S: ReadableStore + QueryableProofStore + TransactionIdentifierLoader> TransactionPreviewer<S> {
    /// Executes the transaction compiled from the given request in a preview mode.
    pub fn preview(
        &self,
        preview_request: PreviewRequest,
    ) -> Result<ProcessedPreviewResult, PreviewError> {
        let read_store = self.store.read();
        let intent = self.create_intent(preview_request, read_store.deref());

        let validator = NotarizedTransactionValidator::new(self.validation_config);
        let validated = validator
            .validate_preview_intent_v1(intent)
            .map_err(PreviewError::TransactionValidationError)?;
        let transaction_logic = self
            .execution_configurator
            .wrap_preview_transaction(&validated);

        let receipt = transaction_logic.execute_on(read_store.deref());
        let substate_changes = match &receipt.transaction_result {
            TransactionResult::Commit(commit) => {
                ProcessedCommitResult::compute_substate_changes::<S, SpreadPrefixKeyMapper>(
                    read_store.deref(),
                    &commit.state_updates.system_updates,
                )
            }
            _ => Vec::new(),
        };

        Ok(ProcessedPreviewResult {
            receipt,
            substate_changes,
        })
    }

    fn create_intent(&self, preview_request: PreviewRequest, read_store: &S) -> PreviewIntentV1 {
        let notary = preview_request.notary_public_key.unwrap_or_else(|| {
            PublicKey::Secp256k1(Secp256k1PrivateKey::from_u64(2).unwrap().public_key())
        });
        let effective_epoch_range = preview_request.explicit_epoch_range.unwrap_or_else(|| {
            let current_epoch = read_store.get_epoch();
            Range {
                start: current_epoch,
                end: current_epoch.after(self.validation_config.max_epoch_range),
            }
        });
        let (instructions, blobs) = preview_request.manifest.for_intent();
        PreviewIntentV1 {
            intent: IntentV1 {
                header: TransactionHeaderV1 {
                    network_id: self.validation_config.network_id,
                    start_epoch_inclusive: effective_epoch_range.start,
                    end_epoch_exclusive: effective_epoch_range.end,
                    nonce: preview_request.nonce,
                    notary_public_key: notary,
                    notary_is_signatory: preview_request.notary_is_signatory,
                    tip_percentage: preview_request.tip_percentage,
                },
                instructions,
                blobs,
                message: preview_request.message,
            },
            signer_public_keys: preview_request.signer_public_keys,
            flags: preview_request.flags,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::jni::state_manager::ActualStateManager;
    use crate::mempool_manager::MempoolManager;
    use crate::priority_mempool::PriorityMempool;
    use crate::store::{DatabaseFlags, InMemoryStore, StateManagerDatabase};
    use crate::transaction::{
        CachedCommittabilityValidator, CommittabilityValidator, ExecutionConfigurator,
        TransactionPreviewer,
    };
    use crate::{
        LoggingConfig, PendingTransactionResultCache, PreviewRequest, StateManager,
        StateManagerLoggingConfig,
    };
    use node_common::config::limits::VertexLimitsConfig;
    use node_common::config::MempoolConfig;
    use node_common::locks::RwLock;
    use prometheus::Registry;
    use radix_engine::transaction::FeeReserveConfig;
    use radix_engine_common::network::NetworkDefinition;
    use std::sync::Arc;
    use transaction::builder::ManifestBuilder;
    use transaction::model::{MessageV1, PreviewFlags};

    #[test]
    fn test_preview_processed_substate_changes() {
        // TODO: extract test state manager setup to a method/helper
        let network = NetworkDefinition::simulator();
        let logging_config = LoggingConfig {
            engine_trace: false,
            state_manager_config: StateManagerLoggingConfig {
                log_on_transaction_rejection: false,
            },
        };
        let database = Arc::new(RwLock::new(StateManagerDatabase::InMemory(
            InMemoryStore::new(DatabaseFlags::default()),
        )));
        let metric_registry = Registry::new();
        let execution_configurator = Arc::new(ExecutionConfigurator::new(
            &logging_config,
            FeeReserveConfig::default(),
        ));
        let pending_transaction_result_cache = Arc::new(RwLock::new(
            PendingTransactionResultCache::new(10000, 10000),
        ));
        let committability_validator = Arc::new(CommittabilityValidator::new(
            &network,
            database.clone(),
            execution_configurator.clone(),
        ));
        let cached_committability_validator = CachedCommittabilityValidator::new(
            database.clone(),
            committability_validator,
            pending_transaction_result_cache.clone(),
        );
        let mempool = Arc::new(RwLock::new(PriorityMempool::new(MempoolConfig {
            max_total_transactions_size: 10 * 1024 * 1024,
            max_transaction_count: 10,
        })));
        let mempool_manager = Arc::new(MempoolManager::new_for_testing(
            mempool,
            cached_committability_validator,
            &metric_registry,
        ));
        let state_manager: Arc<RwLock<ActualStateManager>> =
            Arc::new(RwLock::new(StateManager::new(
                &network,
                VertexLimitsConfig::default(),
                database.clone(),
                mempool_manager,
                execution_configurator.clone(),
                pending_transaction_result_cache,
                logging_config,
                &metric_registry,
            )));

        state_manager.read().execute_genesis_for_unit_tests();

        let transaction_previewer = Arc::new(TransactionPreviewer::new(
            &network,
            database,
            execution_configurator,
        ));

        let preview_manifest = ManifestBuilder::new().lock_fee_from_faucet().build();

        let preview_response = transaction_previewer.preview(PreviewRequest {
            manifest: preview_manifest,
            explicit_epoch_range: None,
            notary_public_key: None,
            notary_is_signatory: true,
            tip_percentage: 0,
            nonce: 0,
            signer_public_keys: vec![],
            flags: PreviewFlags {
                use_free_credit: true,
                assume_all_signature_proofs: true,
                skip_epoch_check: false,
            },
            message: MessageV1::None,
        });

        // just checking that we're getting some processed substate changes back in the response
        assert!(!preview_response.unwrap().substate_changes.is_empty());
    }
}
