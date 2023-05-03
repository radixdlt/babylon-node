use radix_engine_common::data::scrypto::ScryptoDecode;
use radix_engine_common::types::{ModuleId, NodeId, SubstateKey};

use super::{MappingError, ResponseError};
use radix_engine::track::db_key_mapper::{MappedSubstateDatabase, SpreadPrefixKeyMapper};
use state_manager::store::StateManagerDatabase;

#[tracing::instrument(skip_all)]
pub(crate) fn read_mandatory_substate<D: ScryptoDecode>(
    database: &StateManagerDatabase,
    node_id: &NodeId,
    module_id: ModuleId,
    substate_key: &SubstateKey,
) -> Result<D, ResponseError<()>> {
    read_optional_substate(
        database,
        node_id,
        module_id,
        substate_key
    ).ok_or_else(
        || {
            MappingError::MismatchedSubstateId {
                message: format!(
                    "Substate key {substate_key:?} not found under NodeId {node_id:?} and module {module_id:?}"
                ),
            }
            .into()
        },
    )
}

#[tracing::instrument(skip_all)]
pub(crate) fn read_optional_substate<D: ScryptoDecode>(
    database: &StateManagerDatabase,
    node_id: &NodeId,
    module_id: ModuleId,
    substate_key: &SubstateKey,
) -> Option<D> {
    database.get_mapped_substate::<SpreadPrefixKeyMapper, D>(node_id, module_id, substate_key)
}
