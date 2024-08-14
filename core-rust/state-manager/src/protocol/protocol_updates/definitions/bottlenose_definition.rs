use crate::engine_prelude::*;
use crate::protocol::*;
use crate::store::rocks_db::ActualStateManagerDatabase;
use node_common::locks::DbLock;
use std::sync::Arc;

pub struct BottlenoseProtocolUpdateDefinition;

impl ProtocolUpdateDefinition for BottlenoseProtocolUpdateDefinition {
    type Overrides = ();

    fn create_batch_generator(
        &self,
        network: &NetworkDefinition,
        database: Arc<DbLock<ActualStateManagerDatabase>>,
        _overrides: Option<Self::Overrides>,
    ) -> Box<dyn ProtocolUpdateNodeBatchGenerator> {
        Box::new(engine_default_for_network::<BottlenoseSettings>(
            network, database,
        ))
    }
}
