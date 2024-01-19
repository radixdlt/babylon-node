use crate::transaction::FlashTransactionV1;
use crate::{
    ProtocolUpdateFlashTxnCommitter, ProtocolUpdater, StateManagerDatabase,
    UpdatableStateComputerConfig, ANEMONE_PROTOCOL_VERSION,
};
use node_common::locks::StateLock;
use radix_engine::prelude::dec;
use radix_engine::utils::{
    generate_seconds_precision_state_updates, generate_validator_fee_fix_state_updates,
    generate_vm_boot_scrypto_minor_version_state_updates,
};
use radix_engine_common::prelude::NetworkDefinition;
use std::ops::Deref;
use std::sync::Arc;

pub struct AnemoneProtocolUpdater {
    pub network: NetworkDefinition,
}

impl ProtocolUpdater for AnemoneProtocolUpdater {
    fn updatable_config(&self) -> UpdatableStateComputerConfig {
        // TODO(anemone): just a stub for testing
        let mut configurator = UpdatableStateComputerConfig::default(self.network.clone());
        configurator.costing_parameters.usd_price = dec!("25");
        configurator
    }

    fn execute_remaining_state_updates(&self, store: Arc<StateLock<StateManagerDatabase>>) {
        // We're using the new configuration to execute the protocol update
        // transactions (although it's not a requirement).
        let updatable_config = self.updatable_config();
        let mut txn_committer = ProtocolUpdateFlashTxnCommitter::new(
            ANEMONE_PROTOCOL_VERSION.to_string(),
            store.clone(),
            updatable_config.execution_configurator(true), /* No fees for protocol updates */
            updatable_config.ledger_transaction_validator(),
        );

        while let Some(next_batch_idx) = txn_committer.next_committable_batch_idx() {
            match next_batch_idx {
                0 => {
                    // Just a single batch for Anemone, which includes
                    // the following transactions:
                    let flash_txns = {
                        let read_db = store.read_current();
                        vec![
                            FlashTransactionV1 {
                                name: "anemone-validator-fee-fix".to_string(),
                                state_updates: generate_validator_fee_fix_state_updates(
                                    read_db.deref(),
                                ),
                            },
                            FlashTransactionV1 {
                                name: "anemone-seconds-precision".to_string(),
                                state_updates: generate_seconds_precision_state_updates(
                                    read_db.deref(),
                                ),
                            },
                            FlashTransactionV1 {
                                name: "anemone-vm-boot".to_string(),
                                state_updates: generate_vm_boot_scrypto_minor_version_state_updates(
                                ),
                            },
                        ]
                    };
                    txn_committer.commit_flash_batch(flash_txns);
                }
                _ => break,
            }
        }
    }
}
