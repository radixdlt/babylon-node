use crate::prelude::*;

/// Any protocol update beginning `custom-` can have content injected via config.
pub struct CustomProtocolUpdateDefinition;

impl CustomProtocolUpdateDefinition {
    pub const RESERVED_NAME_PREFIX: &'static str = "custom-";

    pub fn subnamed(subname: &str) -> ProtocolVersionName {
        ProtocolVersionName::of(format!("{}{}", Self::RESERVED_NAME_PREFIX, subname)).unwrap()
    }

    pub fn matches(name_string: &str) -> bool {
        name_string.starts_with(Self::RESERVED_NAME_PREFIX)
    }
}

/// This is a slightly different structure to [`ProtocolUpdateNodeBatch`] because the latter
/// was updated, but we needed to keep this in the old-style to avoid breaking any tests by
/// changing the models.
#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub enum CustomProtocolUpdateBatch {
    /// Flash transactions.
    FlashTransactions(Vec<FlashTransactionV1>),

    /// An execution of a single test Scenario.
    Scenario(String),
}

impl ProtocolUpdateDefinition for CustomProtocolUpdateDefinition {
    type Overrides = Vec<CustomProtocolUpdateBatch>;

    fn create_batch_generator(
        &self,
        _context: ProtocolUpdateContext,
        overrides_hash: Option<Hash>,
        overrides: Option<Self::Overrides>,
    ) -> Box<dyn NodeProtocolUpdateGenerator> {
        Box::new(ArbitraryNodeBatchGenerator {
            config_hash: overrides_hash.unwrap_or(Hash([0; Hash::LENGTH])),
            batches: {
                overrides
                    .unwrap_or_default()
                    .into_iter()
                    .map(|batch| match batch {
                        CustomProtocolUpdateBatch::FlashTransactions(transactions) => {
                            let batch = ProtocolUpdateBatch {
                                transactions: transactions.into_iter().map(|t| t.into()).collect(),
                            };
                            NodeProtocolUpdateBatch::ProtocolUpdateBatch(batch)
                        }
                        CustomProtocolUpdateBatch::Scenario(scenario_name) => {
                            NodeProtocolUpdateBatch::Scenario(scenario_name)
                        }
                    })
                    .collect()
            },
        })
    }
}

pub struct ArbitraryNodeBatchGenerator {
    pub config_hash: Hash,
    pub batches: Vec<NodeProtocolUpdateBatch>,
}

impl ArbitraryNodeBatchGenerator {
    pub const BATCH_GROUP_DESCRIPTOR: &'static str = "principal";
}

impl NodeProtocolUpdateGenerator for ArbitraryNodeBatchGenerator {
    fn config_hash(&self) -> Hash {
        self.config_hash
    }

    fn batch_groups(&self) -> Vec<Box<dyn NodeProtocolUpdateBatchGroupGenerator + '_>> {
        let mut batch_group = NodeFixedBatchGroupGenerator::named(Self::BATCH_GROUP_DESCRIPTOR);
        for (index, batch) in self.batches.iter().enumerate() {
            batch_group = batch_group.add_batch(format!("batch-{index:02}"), |_| batch.clone())
        }
        vec![batch_group.build()]
    }
}
