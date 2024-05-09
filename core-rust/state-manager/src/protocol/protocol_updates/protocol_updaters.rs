use crate::engine_prelude::*;

use crate::transaction::FlashTransactionV1;

/// An atomic part of a protocol update.
#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub enum ProtocolUpdateAction {
    /// An explicit batch of flash transactions.
    FlashTransactions(Vec<FlashTransactionV1>),

    /// An execution of a single test Scenario.
    Scenario(String),
}

/// A provider of actions comprising a single protocol update.
/// This is a lazy provider (rather than a [`Vec`]), since e.g. massive flash transactions could
/// overload memory if initialized all at once.
pub trait ProtocolUpdateActionProvider {
    /// Returns an action at the given index.
    /// Panics if the index is out of bounds (as given by the [`Self::action_count()`].
    fn provide_action(&self, index: u32) -> ProtocolUpdateAction;

    /// Returns the number of contained actions.
    fn action_count(&self) -> u32;
}

/// A [`ProtocolUpdateActionProvider`] decorator which additionally executes post-update Scenarios.
pub struct WithScenariosActionProvider<'b, B: ProtocolUpdateActionProvider + ?Sized> {
    pub base_action_provider: &'b B,
    pub scenario_names: Vec<String>,
}

impl<'b, B: ProtocolUpdateActionProvider + ?Sized> ProtocolUpdateActionProvider
    for WithScenariosActionProvider<'b, B>
{
    fn provide_action(&self, index: u32) -> ProtocolUpdateAction {
        let base_action_count = self.base_action_provider.action_count();
        if index < base_action_count {
            self.base_action_provider.provide_action(index)
        } else {
            let scenario_index = index.checked_sub(base_action_count).unwrap();
            let scenario_name = self
                .scenario_names
                .get(scenario_index as usize)
                .unwrap()
                .clone();
            ProtocolUpdateAction::Scenario(scenario_name)
        }
    }

    fn action_count(&self) -> u32 {
        self.base_action_provider
            .action_count()
            .checked_add(u32::try_from(self.scenario_names.len()).unwrap())
            .unwrap()
    }
}
