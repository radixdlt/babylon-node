use crate::engine_prelude::*;
use node_common::locks::DbLock;
use std::ops::Range;
use std::sync::Arc;

use crate::historical_state::{StateHistoryError, VersionScopingSupport};

use crate::transaction::*;
use crate::{
    ActualStateManagerDatabase, GlobalBalanceSummary, LedgerStateChanges, LedgerStateSummary,
    PreviewRequest, ProcessedCommitResult, StateVersion,
};

/// A transaction preview runner.
pub struct TransactionPreviewer {
    database: Arc<DbLock<ActualStateManagerDatabase>>,
    execution_configurator: Arc<ExecutionConfigurator>,
    validation_config: ValidationConfig,
}

pub struct ProcessedPreviewResult {
    pub base_ledger_state: LedgerStateSummary,
    pub receipt: TransactionReceipt,
    pub state_changes: LedgerStateChanges,
    pub global_balance_summary: GlobalBalanceSummary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreviewerError {
    FromEngine(PreviewError),
    FromStateHistory(StateHistoryError),
}

impl TransactionPreviewer {
    pub fn new(
        database: Arc<DbLock<ActualStateManagerDatabase>>,
        execution_configurator: Arc<ExecutionConfigurator>,
        validation_config: ValidationConfig,
    ) -> Self {
        Self {
            database,
            execution_configurator,
            validation_config,
        }
    }
}

impl TransactionPreviewer {
    /// Executes the transaction compiled from the given request in a preview mode.
    pub fn preview(
        &self,
        preview_request: PreviewRequest,
        requested_state_version: Option<StateVersion>,
    ) -> Result<ProcessedPreviewResult, PreviewerError> {
        // Note: we need to access a snapshot even if running against historical version, since we
        // do not want JMT GC to interfere.
        let database = self
            .database
            .snapshot()
            .scoped_at(requested_state_version)?;

        let base_ledger_state = database.at_ledger_state();

        let intent = self.create_intent(preview_request, base_ledger_state.epoch);

        let validator = NotarizedTransactionValidator::new(self.validation_config);
        let validated = validator
            .validate_preview_intent_v1(intent)
            .map_err(PreviewError::TransactionValidationError)?;
        let transaction_logic = self
            .execution_configurator
            .wrap_preview_transaction(&validated);

        let receipt = transaction_logic.execute_on(&database);
        let (state_changes, global_balance_summary) = match &receipt.result {
            TransactionResult::Commit(commit) => {
                let state_changes = ProcessedCommitResult::compute_ledger_state_changes(
                    &database,
                    &commit.state_updates,
                );
                let global_balance_update = ProcessedCommitResult::compute_global_balance_update(
                    &database,
                    &state_changes,
                    &commit.state_update_summary.vault_balance_changes,
                );
                (state_changes, global_balance_update.global_balance_summary)
            }
            _ => (
                LedgerStateChanges::default(),
                GlobalBalanceSummary::default(),
            ),
        };

        Ok(ProcessedPreviewResult {
            base_ledger_state,
            receipt,
            state_changes,
            global_balance_summary,
        })
    }

    fn create_intent(
        &self,
        preview_request: PreviewRequest,
        at_epoch: Epoch, // used only to resolve implicit epoch range (if not explicitly requested)
    ) -> PreviewIntentV1 {
        let notary = preview_request.notary_public_key.unwrap_or_else(|| {
            PublicKey::Secp256k1(Secp256k1PrivateKey::from_u64(2).unwrap().public_key())
        });
        let effective_epoch_range = preview_request
            .explicit_epoch_range
            .unwrap_or_else(|| Range {
                start: at_epoch,
                end: at_epoch
                    .after(self.validation_config.max_epoch_range)
                    .expect("currently calculated max end epoch is outside of valid range"),
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

impl From<PreviewError> for PreviewerError {
    fn from(value: PreviewError) -> Self {
        Self::FromEngine(value)
    }
}

impl From<StateHistoryError> for PreviewerError {
    fn from(value: StateHistoryError) -> Self {
        Self::FromStateHistory(value)
    }
}

#[cfg(test)]
mod tests {

    use crate::engine_prelude::*;
    use crate::{PreviewRequest, StateManagerConfig};

    use crate::test::create_state_manager;

    #[test]
    fn test_preview_processed_substate_changes() {
        let tmp = tempfile::tempdir().unwrap();
        let state_manager = create_state_manager(StateManagerConfig::new_for_testing(
            tmp.path().to_str().unwrap(),
        ));

        state_manager
            .system_executor
            .execute_genesis_for_unit_tests_with_default_config();

        let preview_manifest = ManifestBuilder::new().lock_fee_from_faucet().build();

        let preview_response = state_manager.transaction_previewer.preview(
            PreviewRequest {
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
                    disable_auth: false,
                },
                message: MessageV1::None,
            },
            None,
        );

        // just checking that we're getting some processed substate changes back in the response
        assert!(!preview_response.unwrap().state_changes.is_empty());
    }
}
