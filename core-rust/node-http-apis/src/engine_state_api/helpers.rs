use state_manager::store::traits::*;
use state_manager::store::StateManagerDatabase;
use state_manager::LedgerHeader;

#[tracing::instrument(skip_all)]
pub(crate) fn read_current_ledger_header(database: &StateManagerDatabase) -> LedgerHeader {
    database
        .get_latest_proof()
        .expect("proof for outputted state must exist")
        .ledger_header
}
