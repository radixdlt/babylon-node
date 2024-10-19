use crate::prelude::*;

/// An atomic part of a protocol update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolUpdateNodeBatch {
    /// A batch from the protocol update
    ProtocolUpdateBatch(ProtocolUpdateBatch),

    /// An execution of a single test Scenario.
    Scenario(String),
}

#[derive(Clone)]
pub struct StartStateIdentifiers {
    pub epoch: Epoch,
    pub proposer_timestamp_ms: i64,
    pub state_version: StateVersion,
}

/// A generator of consecutive transaction batches comprising a single protocol update.
/// This is a lazy provider (rather than a [`Vec`]), since e.g. massive flash transactions could
/// overload memory if initialized all at once.
///
/// This is a wrapper around a [`ProtocolUpdateBatchGenerator`] which also allows us
/// running scenarios. A "Node batch" is either an Engine batch, or a single test Scenario to execute.
pub trait ProtocolUpdateNodeBatchGenerator {
    /// A summary of the configuration of this protocol update, to detect divergence.
    fn config_hash(&self) -> Hash;

    fn genesis_start_state(&self) -> Option<StartStateIdentifiers> {
        None
    }

    /// Generate a batch of transactions to be committed atomically with a proof.
    /// *Panics* if the given batch index is outside the range (see [`Self::batch_count()`]).
    ///
    /// It should be assumed that the [`SubstateDatabase`] has *committed all previous batches*.
    /// This ensures that the update is deterministically continuable if the Node shuts down
    /// mid-update.
    fn generate_batch(
        &self,
        batch_group_index: usize,
        batch_index: usize,
    ) -> ProtocolUpdateNodeBatch;

    /// Returns the number of contained batch groups.
    /// Each batch group is a logical grouping of batches.
    /// For example, at genesis, there are three batch groups:
    /// * Bootstrap (Flash + Bootstrap Txn)
    /// * Chunk Execution
    /// * Wrap up
    ///
    /// The [`Self::generate_batch()`] expects the `batch_group_index`
    /// to be in the range `[0, self.batch_group_descriptors().len() - 1]`.
    fn batch_group_descriptors(&self) -> Vec<String>;

    /// Returns the number of contained batches.
    /// For a fixed batch group, [`Self::generate_batch()`] expects `batch_index`
    /// to be in the range `[0, self.batch_count() - 1]`.
    fn batch_count(&self, batch_group_index: usize) -> usize;
}

/// A [`ProtocolUpdateNodeBatchGenerator`] implementation for the actual Engine's protocol updates.
pub struct EngineBatchGenerator<G: ProtocolUpdateBatchGenerator> {
    database: Arc<DbLock<ActualStateManagerDatabase>>,
    engine_batch_generator: G,
    config_hash: Hash,
}

impl<G: ProtocolUpdateBatchGenerator> EngineBatchGenerator<G> {
    pub fn new(
        database: Arc<DbLock<ActualStateManagerDatabase>>,
        engine_batch_generator: G,
        config_hash: Hash,
    ) -> Self {
        Self {
            database,
            engine_batch_generator,
            config_hash,
        }
    }
}

impl<G: ProtocolUpdateBatchGenerator> ProtocolUpdateNodeBatchGenerator for EngineBatchGenerator<G> {
    fn config_hash(&self) -> Hash {
        self.config_hash
    }

    fn generate_batch(
        &self,
        batch_group_index: usize,
        batch_index: usize,
    ) -> ProtocolUpdateNodeBatch {
        let batch = self.engine_batch_generator.generate_batch(
            self.database.lock().deref(),
            batch_group_index,
            batch_index,
        );
        ProtocolUpdateNodeBatch::ProtocolUpdateBatch(batch)
    }

    fn batch_group_descriptors(&self) -> Vec<String> {
        self.engine_batch_generator.batch_group_descriptors()
    }

    fn batch_count(&self, batch_group_index: usize) -> usize {
        self.engine_batch_generator
            .batch_count(self.database.lock().deref(), batch_group_index)
    }
}

/// A [`ProtocolUpdateNodeBatchGenerator`] decorator which additionally executes post-update Scenarios.
pub struct BatchGeneratorWithScenarios<B: ProtocolUpdateNodeBatchGenerator> {
    pub base_batch_generator: B,
    pub scenario_names: Vec<String>,
    pub fixed_config_hash: Option<Hash>,
    pub genesis_start_identifiers: Option<StartStateIdentifiers>,
}

pub fn create_default_generator_with_scenarios<U: UpdateSettings>(
    context: ProtocolUpdateContext,
    overrides_hash: Option<Hash>,
    overrides: Option<U>,
) -> BatchGeneratorWithScenarios<EngineBatchGenerator<U::BatchGenerator>> {
    let scenario_names = context
        .scenario_config
        .to_run_after_protocol_update(context.network, U::protocol_version());
    let config_hash = overrides_hash.unwrap_or(Hash([0; Hash::LENGTH]));
    let config =
        overrides.unwrap_or_else(|| U::all_enabled_as_default_for_network(context.network));
    BatchGeneratorWithScenarios {
        base_batch_generator: EngineBatchGenerator::new(
            context.database.clone(),
            config.create_batch_generator(),
            config_hash,
        ),
        scenario_names,
        fixed_config_hash: None,
        genesis_start_identifiers: None,
    }
}

/// NOTE:
/// With the introduction of batch_groups in Cuttlefish, this has been changed to use a new batch_group
/// for scenarios.
///
/// This would break nodes which update in the middle of executing a previous protocol update,
/// - but I don't imagine this will realistically happen to anyone.
impl<B: ProtocolUpdateNodeBatchGenerator> ProtocolUpdateNodeBatchGenerator
    for BatchGeneratorWithScenarios<B>
{
    fn genesis_start_state(&self) -> Option<StartStateIdentifiers> {
        self.genesis_start_identifiers.clone()
    }

    fn config_hash(&self) -> Hash {
        self.fixed_config_hash.unwrap_or_else(|| {
            let base_hash = self.base_batch_generator.config_hash();
            if !self.scenario_names.is_empty() {
                let scenarios_hash = hash(scrypto_encode(&self.scenario_names).unwrap());
                hash({
                    let mut hash_input = vec![];
                    hash_input.extend_from_slice(base_hash.as_slice());
                    hash_input.extend_from_slice(scenarios_hash.as_slice());
                    hash_input
                })
            } else {
                base_hash
            }
        })
    }

    fn generate_batch(
        &self,
        batch_group_index: usize,
        batch_index: usize,
    ) -> ProtocolUpdateNodeBatch {
        let base_batch_groups_len = self.base_batch_generator.batch_group_descriptors().len();
        match batch_group_index {
            x if x < base_batch_groups_len => self
                .base_batch_generator
                .generate_batch(batch_group_index, batch_index),
            x if x == base_batch_groups_len => {
                let scenario_name = self.scenario_names.get(batch_index).unwrap().clone();
                ProtocolUpdateNodeBatch::Scenario(scenario_name)
            }
            _ => {
                panic!("Invalid batch group index: {batch_group_index}")
            }
        }
    }

    fn batch_group_descriptors(&self) -> Vec<String> {
        let mut base_groups = self.base_batch_generator.batch_group_descriptors();
        if !self.scenario_names.is_empty() {
            base_groups.push("Scenarios".to_string())
        }
        base_groups
    }

    fn batch_count(&self, batch_group_index: usize) -> usize {
        let base_batch_groups_len = self.base_batch_generator.batch_group_descriptors().len();
        match batch_group_index {
            x if x < base_batch_groups_len => {
                self.base_batch_generator.batch_count(batch_group_index)
            }
            x if x == base_batch_groups_len => self.scenario_names.len(),
            _ => {
                panic!("Invalid batch group index: {batch_group_index}")
            }
        }
    }
}
