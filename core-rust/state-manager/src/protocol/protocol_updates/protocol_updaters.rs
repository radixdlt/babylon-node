use crate::prelude::*;

/// An atomic part of a protocol update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeProtocolUpdateBatch {
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
/// This is a wrapper around a [`ProtocolUpdateGenerator`] which also allows us
/// running scenarios. A "Node batch" is either an Engine batch, or a single test Scenario to execute.
pub trait NodeProtocolUpdateGenerator {
    /// A summary of the configuration of this protocol update, to detect divergence.
    fn config_hash(&self) -> Hash;

    fn insert_status_tracking_flash_transactions(&self) -> bool;

    fn genesis_start_state(&self) -> Option<StartStateIdentifiers> {
        None
    }

    /// Return the list of batch groups for the protocol update.
    ///
    /// Each should be a fixed, conceptual step in the update process.
    fn batch_groups(&self) -> Vec<Box<dyn NodeProtocolUpdateBatchGroupGenerator + '_>>;
}

/// Each batch group is a logical grouping of batches.
///
/// For example, at genesis, there are three batch groups:
/// * `"bootstrap"` (flash + bootstrap transaction)
/// * `"chunks"`
/// * `"wrap-up"`
/// * `"scenarios"`
pub trait NodeProtocolUpdateBatchGroupGenerator<'a> {
    /// This is `&'static` because batch groups are intended to be fixed conceptual steps
    /// in the protocol update.
    ///
    /// The batch-group name should be kebab-case for consistency.
    fn batch_group_name(&self) -> &'static str;

    /// The content of these batches must be *fully reproducible* from the state of the store
    /// *before any updates were committed*. This is why we return an array of batch generators.
    ///
    /// If a protocol update needs to do some complicated/inline batch updates to substates, you may need to:
    /// * Have a first batch group where the planned work is saved batch-by-batch to some special partition
    /// * Have a second batch group where the planned work is performed, by reading from this special partition
    /// * Have a third batch group where the planned work is deleted
    fn generate_batches(
        self: Box<Self>,
        store: &dyn SubstateDatabase,
    ) -> Vec<Box<dyn NodeProtocolUpdateBatchGenerator + 'a>>;
}

/// Generate a batch of transactions to be committed atomically with a proof.
///
/// It should be assumed that the [`SubstateDatabase`] has *committed all previous batches*.
/// This ensures that the update is deterministically continuable if the node shuts down
/// mid-update.
pub trait NodeProtocolUpdateBatchGenerator {
    /// The batch name should be kebab-case for consistency
    fn batch_name(&self) -> &str;

    /// Generates the content of the batch
    fn generate_batch(self: Box<Self>, store: &dyn SubstateDatabase) -> NodeProtocolUpdateBatch;
}

/// A node wrapper around a [`ProtocolUpdateGenerator`].
pub struct WrappedProtocolUpdateGenerator {
    engine_generator: Box<dyn ProtocolUpdateGenerator>,
    config_hash: Hash,
}

impl WrappedProtocolUpdateGenerator {
    pub fn new(
        engine_batch_generator: Box<dyn ProtocolUpdateGenerator>,
        config_hash: Hash,
    ) -> Self {
        Self {
            engine_generator: engine_batch_generator,
            config_hash,
        }
    }
}

impl NodeProtocolUpdateGenerator for WrappedProtocolUpdateGenerator {
    fn config_hash(&self) -> Hash {
        self.config_hash
    }

    fn insert_status_tracking_flash_transactions(&self) -> bool {
        self.engine_generator
            .insert_status_tracking_flash_transactions()
    }

    fn batch_groups(&self) -> Vec<Box<dyn NodeProtocolUpdateBatchGroupGenerator + '_>> {
        self.engine_generator
            .batch_groups()
            .into_iter()
            .map(
                |batch_group_generator| -> Box<dyn NodeProtocolUpdateBatchGroupGenerator + '_> {
                    Box::new(WrappedProtocolUpdateBatchGroupGenerator {
                        engine_batch_group_generator: batch_group_generator,
                    })
                },
            )
            .collect()
    }
}

/// A node wrapper around a [`ProtocolUpdateBatchGroupGenerator`].
pub struct WrappedProtocolUpdateBatchGroupGenerator<'a> {
    engine_batch_group_generator: Box<dyn ProtocolUpdateBatchGroupGenerator<'a> + 'a>,
}

impl<'a> NodeProtocolUpdateBatchGroupGenerator<'a>
    for WrappedProtocolUpdateBatchGroupGenerator<'a>
{
    fn batch_group_name(&self) -> &'static str {
        self.engine_batch_group_generator.batch_group_name()
    }

    fn generate_batches(
        self: Box<Self>,
        store: &dyn SubstateDatabase,
    ) -> Vec<Box<dyn NodeProtocolUpdateBatchGenerator + 'a>> {
        self.engine_batch_group_generator
            .generate_batches(store)
            .into_iter()
            .map(
                |batch_generator| -> Box<dyn NodeProtocolUpdateBatchGenerator + 'a> {
                    Box::new(WrappedProtocolUpdateBatchGenerator {
                        engine_batch_generator: batch_generator,
                    })
                },
            )
            .collect()
    }
}

/// A node wrapper around a [`ProtocolUpdateBatchGenerator`].
pub struct WrappedProtocolUpdateBatchGenerator<'a> {
    engine_batch_generator: Box<dyn ProtocolUpdateBatchGenerator + 'a>,
}

impl<'a> NodeProtocolUpdateBatchGenerator for WrappedProtocolUpdateBatchGenerator<'a> {
    fn batch_name(&self) -> &str {
        self.engine_batch_generator.batch_name()
    }

    fn generate_batch(self: Box<Self>, store: &dyn SubstateDatabase) -> NodeProtocolUpdateBatch {
        NodeProtocolUpdateBatch::ProtocolUpdateBatch(
            self.engine_batch_generator.generate_batch(store),
        )
    }
}

/// A [`ProtocolUpdateNodeBatchGenerator`] decorator which additionally executes post-update Scenarios.
pub struct NodeProtocolUpdateWithScenariosGenerator<B: NodeProtocolUpdateGenerator> {
    pub base_batch_generator: B,
    pub scenario_names: Vec<String>,
    pub fixed_config_hash: Option<Hash>,
    pub genesis_start_identifiers: Option<StartStateIdentifiers>,
    pub insert_scenarios_batch_group_at: usize,
}

pub fn create_default_generator_with_scenarios<U: UpdateSettings>(
    context: ProtocolUpdateContext,
    overrides_hash: Option<Hash>,
    overrides: Option<U>,
) -> NodeProtocolUpdateWithScenariosGenerator<WrappedProtocolUpdateGenerator> {
    let scenario_names = context
        .scenario_config
        .to_run_after_protocol_update(context.network, U::protocol_version());
    let config_hash = overrides_hash.unwrap_or(Hash([0; Hash::LENGTH]));
    let config =
        overrides.unwrap_or_else(|| U::all_enabled_as_default_for_network(context.network));
    let base_batch_generator =
        WrappedProtocolUpdateGenerator::new(Box::new(config.create_generator()), config_hash);
    let insert_scenarios_batch_group_at = base_batch_generator.batch_groups().len();
    NodeProtocolUpdateWithScenariosGenerator {
        base_batch_generator,
        scenario_names,
        fixed_config_hash: None,
        genesis_start_identifiers: None,
        insert_scenarios_batch_group_at,
    }
}

/// NOTE:
/// With the introduction of batch_groups in Cuttlefish, this has been changed to use a new batch_group
/// for scenarios.
///
/// This would break nodes which update in the middle of executing a previous protocol update,
/// - but I don't imagine this will realistically happen to anyone.
impl<B: NodeProtocolUpdateGenerator> NodeProtocolUpdateGenerator
    for NodeProtocolUpdateWithScenariosGenerator<B>
{
    fn genesis_start_state(&self) -> Option<StartStateIdentifiers> {
        self.genesis_start_identifiers.clone()
    }

    fn insert_status_tracking_flash_transactions(&self) -> bool {
        self.base_batch_generator
            .insert_status_tracking_flash_transactions()
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

    fn batch_groups(&self) -> Vec<Box<dyn NodeProtocolUpdateBatchGroupGenerator + '_>> {
        let mut batch_groups = self.base_batch_generator.batch_groups();
        if !self.scenario_names.is_empty() {
            batch_groups.insert(
                self.insert_scenarios_batch_group_at,
                Box::new(ScenariosBatchGroupGenerator {
                    scenario_names: self.scenario_names.clone(),
                }),
            );
        }
        batch_groups
    }
}

pub struct ScenariosBatchGroupGenerator {
    scenario_names: Vec<String>,
}

impl<'a> NodeProtocolUpdateBatchGroupGenerator<'a> for ScenariosBatchGroupGenerator {
    fn batch_group_name(&self) -> &'static str {
        "scenarios"
    }

    fn generate_batches(
        self: Box<Self>,
        _store: &dyn SubstateDatabase,
    ) -> Vec<Box<dyn NodeProtocolUpdateBatchGenerator + 'a>> {
        self.scenario_names
            .into_iter()
            .map(
                |scenario| -> Box<dyn NodeProtocolUpdateBatchGenerator + 'a> {
                    Box::new(ScenarioBatchGenerator {
                        batch_name: scenario.to_ascii_lowercase().replace("_", "-"),
                        scenario,
                    })
                },
            )
            .collect()
    }
}

pub struct ScenarioBatchGenerator {
    batch_name: String,
    scenario: String,
}

impl NodeProtocolUpdateBatchGenerator for ScenarioBatchGenerator {
    fn batch_name(&self) -> &str {
        self.batch_name.as_str()
    }

    fn generate_batch(self: Box<Self>, _store: &dyn SubstateDatabase) -> NodeProtocolUpdateBatch {
        NodeProtocolUpdateBatch::Scenario(self.scenario)
    }
}

/// A simple batch group generator, which knows its batches in advance.
///
/// For some protocol updates, you might want to use a custom batch group generator,
/// which is more lazy, or sources its work from the database.
pub struct NodeFixedBatchGroupGenerator<'a> {
    name: &'static str,
    batches: Vec<Box<dyn NodeProtocolUpdateBatchGenerator + 'a>>,
}

impl<'a> NodeFixedBatchGroupGenerator<'a> {
    pub fn named(name: &'static str) -> Self {
        if name != name.to_ascii_lowercase().as_str() {
            panic!("Protocol update batch group names should be in kebab-case for consistency");
        }
        Self {
            name,
            batches: vec![],
        }
    }

    pub fn add_bespoke_batch(mut self, batch: impl NodeProtocolUpdateBatchGenerator + 'a) -> Self {
        self.batches.push(Box::new(batch));
        self
    }

    pub fn add_batch(
        self,
        name: impl Into<String>,
        generator: impl FnOnce(&dyn SubstateDatabase) -> NodeProtocolUpdateBatch + 'a,
    ) -> Self {
        self.add_bespoke_batch(NodeBatchGenerator::new(name, generator))
    }

    pub fn build(self) -> Box<dyn NodeProtocolUpdateBatchGroupGenerator<'a> + 'a> {
        Box::new(self)
    }
}

impl<'a> NodeProtocolUpdateBatchGroupGenerator<'a> for NodeFixedBatchGroupGenerator<'a> {
    fn batch_group_name(&self) -> &'static str {
        self.name
    }

    fn generate_batches(
        self: Box<Self>,
        _store: &dyn SubstateDatabase,
    ) -> Vec<Box<dyn NodeProtocolUpdateBatchGenerator + 'a>> {
        self.batches
    }
}

#[allow(clippy::type_complexity)]
pub struct NodeBatchGenerator<'a> {
    name: String,
    generator: Box<dyn FnOnce(&dyn SubstateDatabase) -> NodeProtocolUpdateBatch + 'a>,
}

impl<'a> NodeBatchGenerator<'a> {
    pub fn new(
        name: impl Into<String>,
        generator: impl FnOnce(&dyn SubstateDatabase) -> NodeProtocolUpdateBatch + 'a,
    ) -> Self {
        let name = name.into();
        if name.to_ascii_lowercase() != name {
            panic!("Protocol update batch names should be in kebab-case for consistency");
        }
        Self {
            name,
            generator: Box::new(generator),
        }
    }

    pub fn build(self) -> Box<dyn NodeProtocolUpdateBatchGenerator + 'a> {
        Box::new(self)
    }
}

impl<'a> NodeProtocolUpdateBatchGenerator for NodeBatchGenerator<'a> {
    fn batch_name(&self) -> &str {
        &self.name
    }

    fn generate_batch(self: Box<Self>, store: &dyn SubstateDatabase) -> NodeProtocolUpdateBatch {
        (self.generator)(store)
    }
}
