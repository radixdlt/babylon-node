use crate::prelude::*;

pub struct CuttlefishProtocolUpdateDefinition;

impl ProtocolUpdateDefinition for CuttlefishProtocolUpdateDefinition {
    type Overrides = ();

    fn create_batch_generator(
        &self,
        network: &NetworkDefinition,
        database: Arc<DbLock<ActualStateManagerDatabase>>,
        _overrides: Option<Self::Overrides>,
    ) -> Box<dyn ProtocolUpdateNodeBatchGenerator> {
        Box::new(engine_default_for_network::<CuttlefishSettings>(
            network, database,
        ))
    }
}
