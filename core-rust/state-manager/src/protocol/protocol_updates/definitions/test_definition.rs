use crate::prelude::*;

/// Any protocol update beginning `test-` just injects a single transaction.
pub struct TestProtocolUpdateDefinition {
    protocol_name: ProtocolVersionName,
}

impl TestProtocolUpdateDefinition {
    pub const RESERVED_NAME_PREFIX: &'static str = "test-";

    pub fn subnamed(subname: &str) -> ProtocolVersionName {
        ProtocolVersionName::of(format!("{}{}", Self::RESERVED_NAME_PREFIX, subname)).unwrap()
    }

    pub fn matches(name_string: &str) -> bool {
        name_string.starts_with(Self::RESERVED_NAME_PREFIX)
    }

    pub fn new(protocol_name: ProtocolVersionName) -> Self {
        if !Self::matches(protocol_name.as_str()) {
            panic!("not a test name");
        }
        Self { protocol_name }
    }
}

impl ProtocolUpdateDefinition for TestProtocolUpdateDefinition {
    type Overrides = ();

    fn create_batch_generator(
        &self,
        _context: ProtocolUpdateContext,
        _overrides_hash: Option<Hash>,
        _overrides: Option<Self::Overrides>,
    ) -> Box<dyn NodeProtocolUpdateGenerator> {
        let batch = ProtocolUpdateBatch {
            transactions: vec![ProtocolUpdateTransaction::flash(
                format!("{}-txn", self.protocol_name),
                StateUpdates::default(),
            )],
        };
        Box::new(ArbitraryNodeBatchGenerator {
            config_hash: Hash([0; Hash::LENGTH]),
            batches: vec![NodeProtocolUpdateBatch::ProtocolUpdateBatch(batch)],
        })
    }
}
