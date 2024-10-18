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
        _network: &NetworkDefinition,
        _database: Arc<DbLock<ActualStateManagerDatabase>>,
        overrides: Option<Self::Overrides>,
    ) -> Box<dyn ProtocolUpdateNodeBatchGenerator> {
        Box::new(ArbitraryNodeBatchGenerator {
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
    pub batches: Vec<ProtocolUpdateNodeBatch>,
}

impl ProtocolUpdateNodeBatchGenerator for ArbitraryNodeBatchGenerator {
    fn generate_batch(&self, batch_idx: usize) -> ProtocolUpdateNodeBatch {
        self.batches.get(batch_idx).unwrap().clone()
    }

    fn batch_count(&self) -> usize {
        self.batches.len()
    }
}
