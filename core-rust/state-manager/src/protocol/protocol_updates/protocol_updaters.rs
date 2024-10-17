use crate::prelude::*;

/// An atomic part of a protocol update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolUpdateNodeBatch {
    /// A batch from the protocol update
    ProtocolUpdateBatch(ProtocolUpdateBatch),

    /// An execution of a single test Scenario.
    Scenario(String),
}

/// A generator of consecutive transaction batches comprising a single protocol update.
/// This is a lazy provider (rather than a [`Vec`]), since e.g. massive flash transactions could
/// overload memory if initialized all at once.
/// Naming note: this is deliberately a "Node batch generator", for disambiguation with the Engine's
/// [`ProtocolUpdateBatchGenerator`] (which is used internally by one of the implementations here).
/// Conceptually, a "Node batch" is "either an Engine batch, or a single test Scenario to execute".
pub trait ProtocolUpdateNodeBatchGenerator {
    /// Returns a batch at the given index.
    /// Panics if the index is out of bounds (as given by the [`Self::batch_count()`].
    fn generate_batch(&self, batch_index: usize) -> ProtocolUpdateNodeBatch;

    /// Returns the number of contained batches.
    fn batch_count(&self) -> usize;
}

/// A [`ProtocolUpdateNodeBatchGenerator`] implementation for the actual Engine's protocol updates.
pub struct EngineBatchGenerator<G> {
    database: Arc<DbLock<ActualStateManagerDatabase>>,
    engine_batch_generator: G,
}

/// Creates an [`EngineBatchGenerator`] for the given [`UpdateSettings`], with all
/// the features that Engine wants enabled by default.
pub fn engine_default_for_network<U: UpdateSettings>(
    network: &NetworkDefinition,
    database: Arc<DbLock<ActualStateManagerDatabase>>,
) -> EngineBatchGenerator<U::BatchGenerator> {
    EngineBatchGenerator {
        database,
        engine_batch_generator: U::all_enabled_as_default_for_network(network)
            .create_batch_generator(),
    }
}

impl<G: ProtocolUpdateBatchGenerator> ProtocolUpdateNodeBatchGenerator for EngineBatchGenerator<G> {
    fn generate_batch(&self, batch_idx: usize) -> ProtocolUpdateNodeBatch {
        let batch =
            self.engine_batch_generator
                .generate_batch(self.database.lock().deref(), 0, batch_idx);
        ProtocolUpdateNodeBatch::ProtocolUpdateBatch(batch)
    }

    fn batch_count(&self) -> usize {
        assert_eq!(
            self.engine_batch_generator.batch_group_descriptors().len(),
            1,
            "Currently the node only supports protocol updates with 1 batch group",
        );
        self.engine_batch_generator.batch_count(0)
    }
}

/// A [`ProtocolUpdateNodeBatchGenerator`] decorator which additionally executes post-update Scenarios.
pub struct WithScenariosNodeBatchGenerator<'b, B: ProtocolUpdateNodeBatchGenerator + ?Sized> {
    pub base_batch_generator: &'b B,
    pub scenario_names: Vec<String>,
}

impl<'b, B: ProtocolUpdateNodeBatchGenerator + ?Sized> ProtocolUpdateNodeBatchGenerator
    for WithScenariosNodeBatchGenerator<'b, B>
{
    fn generate_batch(&self, batch_idx: usize) -> ProtocolUpdateNodeBatch {
        let base_batch_count = self.base_batch_generator.batch_count();
        if batch_idx < base_batch_count {
            self.base_batch_generator.generate_batch(batch_idx)
        } else {
            let scenario_index = batch_idx.checked_sub(base_batch_count).unwrap();
            let scenario_name = self.scenario_names.get(scenario_index).unwrap().clone();
            ProtocolUpdateNodeBatch::Scenario(scenario_name)
        }
    }

    fn batch_count(&self) -> usize {
        self.base_batch_generator
            .batch_count()
            .checked_add(self.scenario_names.len())
            .unwrap()
    }
}
