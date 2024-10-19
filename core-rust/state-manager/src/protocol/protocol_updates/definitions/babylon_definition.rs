use crate::prelude::*;

pub struct BabylonProtocolUpdateDefinition;

impl BabylonProtocolUpdateDefinition {
    fn prepare_raw_genesis_data(genesis_data: JavaGenesisData) -> (BabylonSettings, Vec<String>) {
        let config = genesis_data.initial_config;

        let babylon_settings = BabylonSettings {
            genesis_data_chunks: genesis_data.chunks,
            genesis_epoch: genesis_data.initial_epoch,
            consensus_manager_config: ConsensusManagerConfig {
                max_validators: config.max_validators,
                epoch_change_condition: EpochChangeCondition {
                    min_round_count: config.epoch_min_round_count,
                    max_round_count: config.epoch_max_round_count,
                    target_duration_millis: config.epoch_target_duration_millis,
                },
                num_unstake_epochs: config.num_unstake_epochs,
                total_emission_xrd_per_epoch: config.total_emission_xrd_per_epoch,
                min_validator_reliability: config.min_validator_reliability,
                num_owner_stake_units_unlock_epochs: config.num_owner_stake_units_unlock_epochs,
                num_fee_increase_delay_epochs: config.num_fee_increase_delay_epochs,
                validator_creation_usd_cost: config.validator_creation_usd_cost,
            },
            initial_time_ms: genesis_data.initial_timestamp_ms,
            // Leader gets set to None, to be fixed at the first proper round change.
            initial_current_leader: None,
            faucet_supply: genesis_data.faucet_supply,
        };

        (babylon_settings, genesis_data.scenarios_to_run)
    }
}

#[derive(Debug, ScryptoSbor)]
pub struct JavaGenesisData {
    pub initial_epoch: Epoch,
    pub initial_timestamp_ms: i64,
    pub initial_config: JavaConsensusManagerConfig,
    pub chunks: Vec<GenesisDataChunk>,
    pub faucet_supply: Decimal,
    pub scenarios_to_run: Vec<String>,
}

impl JavaGenesisData {
    pub fn new_from(settings: BabylonSettings, scenarios: Vec<String>) -> Self {
        let config: ConsensusManagerConfig = settings.consensus_manager_config;
        Self {
            initial_epoch: settings.genesis_epoch,
            initial_timestamp_ms: settings.initial_time_ms,
            initial_config: JavaConsensusManagerConfig {
                max_validators: config.max_validators,
                epoch_min_round_count: config.epoch_change_condition.min_round_count,
                epoch_max_round_count: config.epoch_change_condition.max_round_count,
                epoch_target_duration_millis: config.epoch_change_condition.target_duration_millis,
                num_unstake_epochs: config.num_unstake_epochs,
                total_emission_xrd_per_epoch: config.total_emission_xrd_per_epoch,
                min_validator_reliability: config.min_validator_reliability,
                num_owner_stake_units_unlock_epochs: config.num_owner_stake_units_unlock_epochs,
                num_fee_increase_delay_epochs: config.num_fee_increase_delay_epochs,
                validator_creation_usd_cost: config.validator_creation_usd_cost,
            },
            chunks: settings.genesis_data_chunks,
            faucet_supply: settings.faucet_supply,
            scenarios_to_run: scenarios,
        }
    }
}

#[derive(Debug, ScryptoSbor)]
pub struct JavaConsensusManagerConfig {
    pub max_validators: u32,
    pub epoch_min_round_count: u64,
    pub epoch_max_round_count: u64,
    pub epoch_target_duration_millis: u64,
    pub num_unstake_epochs: u64,
    pub total_emission_xrd_per_epoch: Decimal,
    pub min_validator_reliability: Decimal,
    pub num_owner_stake_units_unlock_epochs: u64,
    pub num_fee_increase_delay_epochs: u64,
    pub validator_creation_usd_cost: Decimal,
}

impl ProtocolUpdateDefinition for BabylonProtocolUpdateDefinition {
    type Overrides = JavaGenesisData;

    // We override this to get a more efficient config hash on boot-up to save
    // bringing a large genesis data into memory
    fn config_hash(
        &self,
        context: ProtocolUpdateContext,
        overrides_hash: Option<Hash>,
        _overrides: Option<Self::Overrides>,
    ) -> Hash {
        match overrides_hash {
            Some(hash) => hash,
            None => context.genesis_data_resolver.get_genesis_data_hash(),
        }
    }

    fn create_batch_generator(
        &self,
        context: ProtocolUpdateContext,
        overrides_hash: Option<Hash>,
        overrides: Option<Self::Overrides>,
    ) -> Box<dyn ProtocolUpdateNodeBatchGenerator> {
        let genesis_data = match overrides {
            Some(overrides) => overrides,
            None => {
                let raw_genesis_data = context.genesis_data_resolver.get_raw_genesis_data();
                scrypto_decode(&raw_genesis_data).expect("Could not decode genesis data")
            }
        };

        let config_hash = match overrides_hash {
            Some(hash) => hash,
            None => context.genesis_data_resolver.get_genesis_data_hash(),
        };

        let (babylon_settings, scenario_names) = Self::prepare_raw_genesis_data(genesis_data);
        let genesis_start_identifiers = Some(StartStateIdentifiers {
            epoch: babylon_settings.genesis_epoch,
            proposer_timestamp_ms: babylon_settings.initial_time_ms,
            state_version: StateVersion::pre_genesis(),
        });

        let base_batch_generator = EngineBatchGenerator::new(
            context.database.clone(),
            babylon_settings.create_batch_generator(),
            Hash([0; Hash::LENGTH]), // This hash gets ignored by the fixed outer hash.
        );

        // Insert scenarios before the WrapUp batch group
        let insert_scenarios_at = base_batch_generator
            .batch_group_descriptors()
            .iter()
            .position(|n| n == "WrapUp")
            .expect("Genesis should include the WrapUp batch group");

        Box::new(BatchGeneratorWithScenarios {
            base_batch_generator,
            scenario_names,
            fixed_config_hash: Some(config_hash),
            genesis_start_identifiers,
            insert_scenarios_batch_group_at: insert_scenarios_at,
        })
    }
}
