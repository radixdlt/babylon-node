use crate::engine_prelude::*;

use crate::transaction::FlashTransactionV1;

/// An atomic part of a protocol update.
#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub enum ProtocolUpdateNodeBatch {
    /// An explicit batch of flash transactions.
    FlashTransactions(Vec<FlashTransactionV1>),

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
    fn generate_batch(&self, batch_idx: u32) -> ProtocolUpdateNodeBatch;

    /// Returns the number of contained batches.
    fn batch_count(&self) -> u32;
}

/// A [`ProtocolUpdateNodeBatchGenerator`] decorator which additionally executes post-update Scenarios.
pub struct WithScenariosNodeBatchGenerator<'b, B: ProtocolUpdateNodeBatchGenerator + ?Sized> {
    pub base_batch_generator: &'b B,
    pub scenario_names: Vec<String>,
}

impl<'b, B: ProtocolUpdateNodeBatchGenerator + ?Sized> ProtocolUpdateNodeBatchGenerator
    for WithScenariosNodeBatchGenerator<'b, B>
{
    fn generate_batch(&self, batch_idx: u32) -> ProtocolUpdateNodeBatch {
        let base_batch_count = self.base_batch_generator.batch_count();
        if batch_idx < base_batch_count {
            self.base_batch_generator.generate_batch(batch_idx)
        } else {
            let scenario_index = batch_idx.checked_sub(base_batch_count).unwrap();
            let scenario_name = self
                .scenario_names
                .get(scenario_index as usize)
                .unwrap()
                .clone();
            ProtocolUpdateNodeBatch::Scenario(scenario_name)
        }
    }

    fn batch_count(&self) -> u32 {
        self.base_batch_generator
            .batch_count()
            .checked_add(u32::try_from(self.scenario_names.len()).unwrap())
            .unwrap()
    }
}
