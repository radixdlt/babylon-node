use crate::protocol::*;
use crate::transaction::FlashTransactionV1;
use radix_engine::types::*;
use radix_engine::utils::{
    generate_pools_v1_1_state_updates, generate_seconds_precision_state_updates,
    generate_validator_fee_fix_state_updates, generate_vm_boot_scrypto_minor_version_state_updates,
};
use radix_engine_store_interface::interface::SubstateDatabase;

pub struct AnemoneProtocolUpdateDefinition;

impl ProtocolUpdateDefinition for AnemoneProtocolUpdateDefinition {
    type Overrides = ();

    fn create_updater(
        new_protocol_version: &ProtocolVersionName,
        network_definition: &NetworkDefinition,
        _config: Option<Self::Overrides>,
    ) -> Box<dyn ProtocolUpdater> {
        Box::new(BatchedUpdater::new(
            new_protocol_version.clone(),
            Self::state_computer_config(network_definition),
            AnemoneBatchGenerator,
        ))
    }

    fn state_computer_config(
        network_definition: &NetworkDefinition,
    ) -> ProtocolStateComputerConfig {
        ProtocolStateComputerConfig::default(network_definition.clone())
    }
}

struct AnemoneBatchGenerator;

impl UpdateBatchGenerator for AnemoneBatchGenerator {
    fn generate_batch(
        &self,
        store: &impl SubstateDatabase,
        batch_index: u32,
    ) -> Option<Vec<UpdateTransaction>> {
        match batch_index {
            0 => {
                // Just a single batch for Anemone, which includes the following transactions:
                Some(vec![
                    FlashTransactionV1 {
                        name: "anemone-validator-fee-fix".to_string(),
                        state_updates: generate_validator_fee_fix_state_updates(store),
                    }
                    .into(),
                    FlashTransactionV1 {
                        name: "anemone-seconds-precision".to_string(),
                        state_updates: generate_seconds_precision_state_updates(store),
                    }
                    .into(),
                    FlashTransactionV1 {
                        name: "anemone-vm-boot".to_string(),
                        state_updates: generate_vm_boot_scrypto_minor_version_state_updates(),
                    }
                    .into(),
                    FlashTransactionV1 {
                        name: "anemone-pools".to_string(),
                        state_updates: generate_pools_v1_1_state_updates(store),
                    }
                    .into(),
                ])
            }
            _ => None,
        }
    }
}
