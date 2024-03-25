use state_manager::store::traits::*;
use state_manager::store::StateManagerDatabase;
use state_manager::{LedgerHeader, ReadableRocks, StateVersion};

#[tracing::instrument(skip_all)]
pub(crate) fn read_current_ledger_header(
    database: &StateManagerDatabase<impl ReadableRocks>,
) -> LedgerHeader {
    database
        .get_latest_proof()
        .expect("proof for outputted state must exist")
        .ledger_header
}

pub(crate) fn read_effective_ledger_header(
    database: &StateManagerDatabase<impl ReadableRocks>,
    requested_state_version: Option<StateVersion>,
) -> LedgerHeader {
    requested_state_version
        .map(|state_version| database.get_proof_iter(state_version).next())
        .unwrap_or_else(|| database.get_latest_proof())
        .expect("proof for outputted state must exist")
        .ledger_header
}
