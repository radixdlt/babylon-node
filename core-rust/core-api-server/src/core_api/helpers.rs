use radix_engine_common::data::scrypto::ScryptoDecode;
use radix_engine_common::types::{ModuleId, NodeId, SubstateKey};

use serde::Serialize;
use state_manager::store::StateManagerDatabase;
use std::io::Write;

use super::{MappingError, ResponseError};
use radix_engine::track::db_key_mapper::{MappedSubstateDatabase, SpreadPrefixKeyMapper};

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

struct ByteCountWriter<'a> {
    bytes: &'a mut usize,
}

impl<'a> ByteCountWriter<'a> {
    fn new(bytes: &'a mut usize) -> Self {
        Self { bytes }
    }
}

impl<'a> Write for ByteCountWriter<'a> {
    fn write(&mut self, data: &[u8]) -> Result<usize, std::io::Error> {
        *self.bytes += data.len();
        Ok(data.len())
    }

    fn flush(&mut self) -> Result<(), std::io::Error> {
        Ok(())
    }
}

pub trait GetJsonSize: Serialize {
    fn get_json_size(&self) -> usize {
        let mut bytes = 0;
        {
            let writer = ByteCountWriter::new(&mut bytes);
            serde_json::to_writer(writer, &self).expect("Failed to serialize JSON");
        }
        bytes
    }
}

impl<T> GetJsonSize for T where T: Serialize {}
