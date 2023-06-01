use parking_lot::RwLock;
use radix_engine::track::db_key_mapper::SpreadPrefixKeyMapper;
use radix_engine::transaction::{PreviewError, TransactionReceipt};
use radix_engine_common::types::Epoch;
use std::ops::{Deref, Range};
use std::sync::Arc;
use std::time::Duration;

use crate::query::{StateManagerSubstateQueries, TransactionIdentifierLoader};
use crate::staging::{HashUpdateContext, ProcessedTransactionReceipt, ReadableStore};
use crate::store::traits::QueryableProofStore;
use crate::transaction::*;
use crate::{EpochTransactionIdentifiers, PreviewRequest};
use radix_engine_common::prelude::*;
use transaction::ecdsa_secp256k1::EcdsaSecp256k1PrivateKey;
use transaction::model::*;
use transaction::validation::NotarizedTransactionValidator;
use transaction::validation::ValidationConfig;

const PREVIEW_RUNTIME_WARN_THRESHOLD: Duration = Duration::from_millis(500);

/// A transaction preview runner.
pub struct TransactionPreviewer<S> {
    store: Arc<RwLock<S>>,
    execution_configurator: Arc<ExecutionConfigurator>,
    validation_config: ValidationConfig,
}

pub struct ProcessedPreviewResult {
    pub receipt: TransactionReceipt,
    pub processed_receipt: ProcessedTransactionReceipt,
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

        let parent_accumulator_state = read_store.get_top_accumulator_state();
        let epoch_transaction_identifiers = read_store
            .get_last_epoch_proof()
            .map(|epoch_proof| EpochTransactionIdentifiers::from(&epoch_proof.ledger_header))
            .unwrap_or_else(EpochTransactionIdentifiers::pre_genesis);

        let validator = NotarizedTransactionValidator::new(self.validation_config);
        let validated = validator
            .validate_preview_intent_v1(intent)
            .map_err(PreviewError::TransactionValidationError)?;
        let transaction_logic = self
            .execution_configurator
            .wrap(validated.get_executable(), ConfigType::Preview)
            .warn_after(PREVIEW_RUNTIME_WARN_THRESHOLD, "preview");
        let receipt = transaction_logic.execute_on(read_store.deref());

        // Fake a LedgerPayloadHash for the purposes of mapping the receipt as it doesn't matter for preview
        // TODO - don't do most of this work for preview
        let fake_ledger_hash = LegacyLedgerPayloadHash::from_hash(validated.intent.summary.hash);
        let processed_receipt = ProcessedTransactionReceipt::process::<_, SpreadPrefixKeyMapper>(
            HashUpdateContext {
                store: read_store.deref(),
                epoch_transaction_identifiers: &epoch_transaction_identifiers,
                parent_accumulator_state: &parent_accumulator_state,
                legacy_payload_hash: &fake_ledger_hash,
            },
            receipt.clone(),
        );

        Ok(ProcessedPreviewResult {
            receipt,
            processed_receipt,
        })
    }

    fn create_intent(&self, preview_request: PreviewRequest, read_store: &S) -> PreviewIntentV1 {
        let notary = preview_request.notary_public_key.unwrap_or_else(|| {
            PublicKey::EcdsaSecp256k1(EcdsaSecp256k1PrivateKey::from_u64(2).unwrap().public_key())
        });
        let effective_epoch_range = preview_request.explicit_epoch_range.unwrap_or_else(|| {
            let current_epoch = read_store.get_epoch();
            Range {
                start: current_epoch.number(),
                end: current_epoch.number() + self.validation_config.max_epoch_range,
            }
        });
        let (instructions, blobs) = preview_request.manifest.for_intent();
        PreviewIntentV1 {
            intent: IntentV1 {
                header: TransactionHeaderV1 {
                    network_id: self.validation_config.network_id,
                    start_epoch_inclusive: Epoch::of(effective_epoch_range.start),
                    end_epoch_exclusive: Epoch::of(effective_epoch_range.end),
                    nonce: preview_request.nonce,
                    notary_public_key: notary,
                    notary_is_signatory: preview_request.notary_is_signatory,
                    tip_percentage: preview_request.tip_percentage,
                },
                instructions,
                blobs,
                attachments: AttachmentsV1 {},
            },
            signer_public_keys: preview_request.signer_public_keys,
            flags: PreviewFlags {
                unlimited_loan: preview_request.flags.unlimited_loan,
                assume_all_signature_proofs: preview_request.flags.assume_all_signature_proofs,
                permit_duplicate_intent_hash: preview_request.flags.permit_duplicate_intent_hash,
                permit_invalid_header_epoch: preview_request.flags.permit_invalid_header_epoch,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::jni::state_manager::ActualStateManager;
    use crate::mempool_manager::MempoolManager;
    use crate::simple_mempool::SimpleMempool;
    use crate::store::{DatabaseFlags, InMemoryStore, StateManagerDatabase};
    use crate::transaction::{
        CachedCommitabilityValidator, CommitabilityValidator, ExecutionConfigurator,
        TransactionPreviewer,
    };
    use crate::{
        LoggingConfig, MempoolConfig, PendingTransactionResultCache, PreviewRequest, StateManager,
        StateManagerLoggingConfig,
    };
    use parking_lot::RwLock;
    use prometheus::Registry;
    use radix_engine_common::network::NetworkDefinition;
    use radix_engine_common::{dec, manifest_args};
    use radix_engine_interface::constants::FAUCET;
    use std::sync::Arc;
    use transaction::builder::ManifestBuilder;
    use transaction::model::PreviewFlags;

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
        let database = Arc::new(parking_lot::const_rwlock(StateManagerDatabase::InMemory(
            InMemoryStore::new(DatabaseFlags::default()),
        )));
        let metric_registry = Registry::new();
        let execution_configurator = Arc::new(ExecutionConfigurator::new(&logging_config));
        let pending_transaction_result_cache = Arc::new(parking_lot::const_rwlock(
            PendingTransactionResultCache::new(10000, 10000),
        ));
        let commitability_validator = Arc::new(CommitabilityValidator::new(
            &network,
            database.clone(),
            execution_configurator.clone(),
        ));
        let cached_commitability_validator = CachedCommitabilityValidator::new(
            database.clone(),
            commitability_validator,
            pending_transaction_result_cache.clone(),
        );
        let mempool = Arc::new(parking_lot::const_rwlock(SimpleMempool::new(
            MempoolConfig { max_size: 10 },
        )));
        let mempool_manager = Arc::new(MempoolManager::new_for_testing(
            mempool,
            cached_commitability_validator,
            &metric_registry,
        ));
        let state_manager: Arc<RwLock<ActualStateManager>> =
            Arc::new(parking_lot::const_rwlock(StateManager::new(
                &network,
                database.clone(),
                mempool_manager,
                execution_configurator.clone(),
                pending_transaction_result_cache,
                logging_config,
                &metric_registry,
            )));

        state_manager.read().execute_test_genesis();

        let transaction_previewer = Arc::new(TransactionPreviewer::new(
            &network,
            database,
            execution_configurator,
        ));

        let preview_manifest = ManifestBuilder::new()
            .call_method(FAUCET, "lock_fee", manifest_args!(dec!("100")))
            .build();

        let preview_response = transaction_previewer.preview(PreviewRequest {
            manifest: preview_manifest,
            explicit_epoch_range: None,
            notary_public_key: None,
            notary_is_signatory: true,
            tip_percentage: 0,
            nonce: 0,
            signer_public_keys: vec![],
            flags: PreviewFlags {
                unlimited_loan: true,
                assume_all_signature_proofs: true,
                permit_duplicate_intent_hash: false,
                permit_invalid_header_epoch: false,
            },
        });

        // just checking that we're getting some processed substate changes back in the response
        assert!(!preview_response
            .unwrap()
            .processed_receipt
            .expect_commit("".to_string())
            .local_receipt
            .on_ledger
            .substate_changes
            .is_empty());
    }
}
