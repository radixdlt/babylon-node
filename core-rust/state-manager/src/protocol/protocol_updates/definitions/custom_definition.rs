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
    ) -> Box<dyn ProtocolUpdateNodeBatchGenerator> {
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
                            ProtocolUpdateNodeBatch::ProtocolUpdateBatch(batch)
                        }
                        CustomProtocolUpdateBatch::Scenario(scenario_name) => {
                            ProtocolUpdateNodeBatch::Scenario(scenario_name)
                        }
                    })
                    .collect()
            },
        })
    }
}

pub struct ArbitraryNodeBatchGenerator {
    pub config_hash: Hash,
    pub batches: Vec<ProtocolUpdateNodeBatch>,
}

impl ArbitraryNodeBatchGenerator {
    pub const BATCH_DESCRIPTOR: &'static str = "SingleBatch";
}

impl ProtocolUpdateNodeBatchGenerator for ArbitraryNodeBatchGenerator {
    fn config_hash(&self) -> Hash {
        self.config_hash
    }

    fn generate_batch(
        &self,
        batch_group_index: usize,
        batch_index: usize,
    ) -> ProtocolUpdateNodeBatch {
        match batch_group_index {
            0 => self.batches.get(batch_index).unwrap().clone(),
            _ => panic!("Incorrect batch_group_index: {batch_group_index}"),
        }
    }

    fn batch_group_descriptors(&self) -> Vec<String> {
        vec![Self::BATCH_DESCRIPTOR.to_string()]
    }

    fn batch_count(&self, batch_group_index: usize) -> usize {
        match batch_group_index {
            0 => self.batches.len(),
            _ => panic!("Incorrect batch_group_index: {batch_group_index}"),
        }
    }
}
