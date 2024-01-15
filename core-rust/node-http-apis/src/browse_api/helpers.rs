use radix_engine::types::*;

use radix_engine::system::system_substates::{FieldSubstate, KeyValueEntrySubstate};
use radix_engine_interface::api::CollectionIndex;
use radix_engine_store_interface::{db_key_mapper::*, interface::SubstateDatabase};
use serde::Serialize;
use state_manager::store::traits::*;
use state_manager::store::StateManagerDatabase;
use state_manager::LedgerHeader;
use std::io::Write;

use super::*;

#[tracing::instrument(skip_all)]
pub(crate) fn read_current_ledger_header(database: &StateManagerDatabase) -> LedgerHeader {
    database
        .get_last_proof()
        .expect("proof for outputted state must exist")
        .ledger_header
}
