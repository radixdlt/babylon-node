use radix_engine::updates::{ProtocolUpdateStatusSummary, ProtocolUpdateStatusSummarySubstate};

use super::super::*;

use crate::core_api::models;
use crate::prelude::*;

pub fn to_api_protocol_update_status_substate(
    _context: &MappingContext,
    substate: &ProtocolUpdateStatusSummarySubstate,
) -> Result<models::Substate, MappingError> {
    // TODO:CUTTLEFISH
    let ProtocolUpdateStatusSummary {
        protocol_version: _,
        update_status: _,
    } = substate.as_unique_version();

    Ok(models::Substate::ProtocolUpdateStatusModuleFieldSummarySubstate { is_locked: false })
}
