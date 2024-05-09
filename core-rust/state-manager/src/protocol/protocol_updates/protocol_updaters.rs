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
    /// Returns an action at the given index, or [`None`] if the index is out of bounds.
    fn provide_action(&self, index: u32) -> Option<ProtocolUpdateAction>;
}
