use radix_engine::types::*;

use radix_engine_interface::api::CollectionIndex;
use serde::Serialize;
use state_manager::store::StateManagerDatabase;
use std::io::Write;

use super::{MappingError, ResponseError};
use radix_engine_store_interface::db_key_mapper::*;

#[tracing::instrument(skip_all)]
pub(crate) fn read_mandatory_main_field_substate<D: ScryptoDecode>(
    database: &StateManagerDatabase,
    node_id: &NodeId,
    substate_key: &SubstateKey,
) -> Result<D, ResponseError<()>> {
    read_mandatory_substate(database, node_id, OBJECT_BASE_PARTITION, substate_key)
}

#[tracing::instrument(skip_all)]
pub(crate) fn read_mandatory_substate<D: ScryptoDecode>(
    database: &StateManagerDatabase,
    node_id: &NodeId,
    partition_number: PartitionNumber,
    substate_key: &SubstateKey,
) -> Result<D, ResponseError<()>> {
    read_optional_substate(
        database,
        node_id,
        partition_number,
        substate_key
    ).ok_or_else(
        || {
            MappingError::MismatchedSubstateId {
                message: format!(
                    "Substate key {substate_key:?} not found under NodeId {node_id:?} and partition number {partition_number:?}"
                ),
            }
            .into()
        },
    )
}

#[tracing::instrument(skip_all)]
pub(crate) fn read_optional_main_field_substate<D: ScryptoDecode>(
    database: &StateManagerDatabase,
    node_id: &NodeId,
    substate_key: &SubstateKey,
) -> Option<D> {
    read_optional_substate(database, node_id, OBJECT_BASE_PARTITION, substate_key)
}

#[tracing::instrument(skip_all)]
pub(crate) fn read_optional_collection_substate<D: ScryptoDecode + Debug>(
    database: &StateManagerDatabase,
    node_id: &NodeId,
    collection_index: CollectionIndex,
    substate_key: &SubstateKey,
) -> Option<D> {
    // Note - the field partition (if it exists) takes the first partition number,
    // the collections go after - so start at offset 1
    // (assuming there is a tuple partition on the node...)
    let partition_number = OBJECT_BASE_PARTITION
        .at_offset(PartitionOffset(1 + collection_index))
        .unwrap();
    read_optional_substate(database, node_id, partition_number, substate_key)
}

#[tracing::instrument(skip_all)]
pub(crate) fn read_optional_substate<D: ScryptoDecode>(
    database: &StateManagerDatabase,
    node_id: &NodeId,
    partition_number: PartitionNumber,
    substate_key: &SubstateKey,
) -> Option<D> {
    database.get_mapped::<SpreadPrefixKeyMapper, D>(node_id, partition_number, substate_key)
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
