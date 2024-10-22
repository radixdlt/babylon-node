use radix_engine::updates::*;

use super::super::*;

use crate::core_api::models;
use crate::prelude::*;

pub fn to_api_protocol_update_status_substate(
    _context: &MappingContext,
    substate: &ProtocolUpdateStatusSummarySubstate,
) -> Result<models::Substate, MappingError> {
    // TODO:CUTTLEFISH
    let ProtocolUpdateStatusSummary {
        protocol_version,
        update_status,
    } = substate.as_unique_version();

    let update_status = match update_status {
        ProtocolUpdateStatus::Complete => {
            models::ProtocolUpdateStatus::CompleteProtocolUpdateStatus {}
        }
        ProtocolUpdateStatus::InProgress { latest_commit } => {
            models::ProtocolUpdateStatus::InProgressProtocolUpdateStatus {
                latest_commit: Box::new(models::ProtocolUpdateStatusLatestCommit {
                    batch_group_index: to_api_index_as_i64(latest_commit.batch_group_index)?,
                    batch_group_name: latest_commit.batch_group_name.to_string(),
                    batch_index: to_api_index_as_i64(latest_commit.batch_index)?,
                    batch_name: latest_commit.batch_name.to_string(),
                }),
            }
        }
    };

    Ok(
        models::Substate::ProtocolUpdateStatusModuleFieldSummarySubstate {
            is_locked: false,
            update_status: Box::new(update_status),
            protocol_version: ProtocolVersionName::for_engine(*protocol_version).to_string(),
        },
    )
}
