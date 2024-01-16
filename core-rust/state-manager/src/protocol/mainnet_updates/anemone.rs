use crate::flash_templates::consensus_manager_config_flash;
use crate::query::StateManagerSubstateQueries;
use crate::{
    ProtocolUpdateFlashTxnCommitter, ProtocolUpdater, StateComputerConfigurator,
    StateManagerDatabase, StateUpdateExecutor, ANEMONE_PROTOCOL_VERSION,
};
use node_common::locks::StateLock;
use radix_engine::prelude::dec;
use radix_engine::utils::{generate_seconds_precision_state_updates, generate_vm_boot_scrypto_minor_version_state_updates};
use radix_engine_common::prelude::NetworkDefinition;
use std::ops::Deref;
use std::sync::Arc;

pub struct AnemoneProtocolUpdater {
    pub network: NetworkDefinition,
    pub store: Arc<StateLock<StateManagerDatabase>>,
}

impl ProtocolUpdater for AnemoneProtocolUpdater {
    fn protocol_version_name(&self) -> String {
        ANEMONE_PROTOCOL_VERSION.to_string()
    }

    fn state_computer_configurator(&self) -> StateComputerConfigurator {
        // TODO(anemone): just a stub for testing
        let mut configurator = StateComputerConfigurator::default(self.network.clone());
        configurator.costing_parameters.usd_price = dec!("25");
        configurator
    }

    fn state_update_executor(&self) -> Box<dyn StateUpdateExecutor> {
        Box::new(AnemoneStateUpdateExecutor {
            store: self.store.clone(),
            state_computer_configurator: self.state_computer_configurator(),
        })
    }
}

struct AnemoneStateUpdateExecutor {
    store: Arc<StateLock<StateManagerDatabase>>,
    state_computer_configurator: StateComputerConfigurator,
}

impl StateUpdateExecutor for AnemoneStateUpdateExecutor {
    fn execute_remaining_state_updates(&self) {
        let mut txn_committer = ProtocolUpdateFlashTxnCommitter::new(
            ANEMONE_PROTOCOL_VERSION.to_string(),
            self.store.clone(),
            self.state_computer_configurator.clone(),
        );

        while let Some(next_batch_idx) = txn_committer.next_committable_batch_idx() {
            match next_batch_idx {
                0 => {
                    // Batch 0: flash consensus manager config update
                    let mut config = self.store.read_current().get_consensus_manager_config();
                    config.validator_creation_usd_cost = dec!("100");
                    txn_committer.commit_flash(consensus_manager_config_flash(config));
                }
                1 => {
                    // Batch 1: flash seconds precision
                    let state_updates =
                        generate_seconds_precision_state_updates(self.store.read_current().deref());
                    txn_committer.commit_flash(state_updates);
                }
                2 => {
                    // Batch 2: flash VM boot
                    let state_updates =
                        generate_vm_boot_scrypto_minor_version_state_updates();
                    txn_committer.commit_flash(state_updates);
                }
                _ => break,
            }
        }
    }
}
