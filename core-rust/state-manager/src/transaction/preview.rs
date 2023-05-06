use parking_lot::RwLock;
use radix_engine::track::db_key_mapper::SpreadPrefixKeyMapper;
use radix_engine::transaction::{PreviewError, TransactionReceipt};
use std::ops::{Deref, Range};
use std::sync::Arc;
use std::time::Duration;

use crate::query::{StateManagerSubstateQueries, TransactionIdentifierLoader};
use crate::staging::{HashUpdateContext, ProcessedTransactionReceipt, ReadableStore};
use crate::store::traits::QueryableProofStore;
use crate::transaction::{
    ConfigType, ExecutionConfigurator, NoopIntentHashManager, TransactionLogic,
};
use crate::{EpochTransactionIdentifiers, LedgerPayloadHash, PreviewRequest};
use radix_engine_common::crypto::PublicKey;
use radix_engine_common::network::NetworkDefinition;
use transaction::ecdsa_secp256k1::EcdsaSecp256k1PrivateKey;
use transaction::model::{PreviewFlags, PreviewIntent, TransactionHeader, TransactionIntent};
use transaction::validation::NotarizedTransactionValidator;
use transaction::validation::ValidationConfig;

const PREVIEW_RUNTIME_WARN_THRESHOLD: Duration = Duration::from_millis(500);

/// A transaction preview runner.
pub struct TransactionPreviewer<S> {
    store: Arc<RwLock<S>>,
    execution_configurator: Arc<ExecutionConfigurator>,
    validation_config: ValidationConfig,
}

// TODO: Engine preview executor (execute_preview) isn't really used...? remove it?
pub struct PreviewResult {
    pub intent: PreviewIntent,
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
    pub fn preview(&self, preview_request: PreviewRequest) -> Result<PreviewResult, PreviewError> {
        let read_store = self.store.read();
        let intent = self.create_intent(preview_request, read_store.deref());

        let transaction_identifiers = read_store.get_top_transaction_identifiers();
        let epoch_identifiers = read_store
            .get_last_epoch_proof()
            .map(|epoch_proof| EpochTransactionIdentifiers::from(epoch_proof.ledger_header))
            .unwrap_or_else(EpochTransactionIdentifiers::pre_genesis);

        let validator = NotarizedTransactionValidator::new(self.validation_config);
        let executable = validator
            .validate_preview_intent(&intent, &NoopIntentHashManager {})
            .map_err(PreviewError::TransactionValidationError)?;
        let transaction_logic = self
            .execution_configurator
            .wrap(executable, ConfigType::Preview)
            .warn_after(PREVIEW_RUNTIME_WARN_THRESHOLD, "preview");
        let receipt = transaction_logic.execute_on(read_store.deref());

        // Using intent hash as transaction hash for the hash update context; doesn't matter for preview
        let transaction_hash =
            LedgerPayloadHash::for_ledger_payload_bytes(intent.to_bytes().unwrap().as_ref());
        let processed_receipt = ProcessedTransactionReceipt::process::<_, SpreadPrefixKeyMapper>(
            HashUpdateContext {
                store: read_store.deref(),
                epoch_transaction_identifiers: &epoch_identifiers,
                parent_transaction_identifiers: &transaction_identifiers,
                transaction_hash: &transaction_hash,
            },
            receipt.clone(),
        );

        Ok(PreviewResult {
            intent,
            receipt,
            processed_receipt,
        })
    }

    fn create_intent(&self, preview_request: PreviewRequest, read_store: &S) -> PreviewIntent {
        let notary = preview_request.notary_public_key.unwrap_or_else(|| {
            PublicKey::EcdsaSecp256k1(EcdsaSecp256k1PrivateKey::from_u64(2).unwrap().public_key())
        });
        let effective_epoch_range = preview_request.explicit_epoch_range.unwrap_or_else(|| {
            let current_epoch = read_store.get_epoch();
            Range {
                start: current_epoch,
                end: current_epoch + self.validation_config.max_epoch_range,
            }
        });
        PreviewIntent {
            intent: TransactionIntent {
                header: TransactionHeader {
                    version: 1,
                    network_id: self.validation_config.network_id,
                    start_epoch_inclusive: effective_epoch_range.start,
                    end_epoch_exclusive: effective_epoch_range.end,
                    nonce: preview_request.nonce,
                    notary_public_key: notary,
                    notary_as_signatory: preview_request.notary_as_signatory,
                    cost_unit_limit: preview_request.cost_unit_limit,
                    tip_percentage: preview_request.tip_percentage,
                },
                manifest: preview_request.manifest,
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
    use radix_engine::system::bootstrap::{
        GenesisDataChunk, GenesisStakeAllocation, GenesisValidator,
    };
    use radix_engine_common::crypto::EcdsaSecp256k1PublicKey;
    use radix_engine_common::network::NetworkDefinition;
    use radix_engine_common::types::ComponentAddress;
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
        let mempool_manager = Arc::new(MempoolManager::new(
            mempool,
            None,
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

        let genesis_validator: GenesisValidator = EcdsaSecp256k1PublicKey([0; 33]).into();
        let genesis_chunks = vec![
            GenesisDataChunk::Validators(vec![genesis_validator.clone()]),
            GenesisDataChunk::Stakes {
                accounts: vec![ComponentAddress::virtual_account_from_public_key(
                    &genesis_validator.key,
                )],
                allocations: vec![(
                    genesis_validator.key,
                    vec![GenesisStakeAllocation {
                        account_index: 0,
                        xrd_amount: dec!("100"),
                    }],
                )],
            },
        ];

        state_manager
            .read()
            .execute_genesis(genesis_chunks, 0, 100, 10, 10);

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
            notary_as_signatory: true,
            cost_unit_limit: 100000000,
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
