use prometheus::core::*;
use prometheus::*;

// TODO: Can move these into separate modules if we wish to, as long as they all register to the same registry
pub struct StateManagerMetrics {
    pub ledger_state_version: IntGauge,
    pub ledger_transactions_committed: IntCounter,
    pub ledger_last_update_timestamp_ms: IntGauge,
    pub mempool_current_transactions_total: IntGauge,
    pub mempool_submission_added_count: IntCounterVec,
    pub mempool_submission_rejected_count: IntCounterVec,
}

impl StateManagerMetrics {
    pub fn new() -> Self {
        Self {
            ledger_transactions_committed: IntCounter::with_opts(opts!(
                "ledger_transactions_committed",
                "Number of transactions committed to the ledger."
            ))
            .unwrap(),
            ledger_last_update_timestamp_ms: IntGauge::with_opts(opts!(
                "ledger_last_update_timestamp_ms",
                "Last time the ledger was updated."
            ))
            .unwrap(),
            ledger_state_version: IntGauge::with_opts(opts!(
                "ledger_state_version",
                "Version of the ledger state."
            ))
            .unwrap(),
            mempool_current_transactions_total: IntGauge::with_opts(opts!(
                "mempool_current_transactions_total",
                "Number of the transactions in progress in the mempool."
            ))
            .unwrap(),
            mempool_submission_added_count: IntCounterVec::new(
                opts!(
                    "mempool_submission_added_count",
                    "Number of submissions added to the mempool."
                ),
                &["Source"],
            )
            .unwrap(),
            mempool_submission_rejected_count: IntCounterVec::new(
                opts!(
                    "mempool_submission_rejected_count",
                    "Number of the submissions rejected by the mempool."
                ),
                &["Source", "RejectionReason"],
            )
            .unwrap(),
        }
    }

    pub fn register_with(&self, registry: &Registry) {
        let metrics: Vec<Box<dyn Collector>> = vec![
            Box::new(self.ledger_state_version.clone()),
            Box::new(self.ledger_transactions_committed.clone()),
            Box::new(self.ledger_last_update_timestamp_ms.clone()),
            Box::new(self.mempool_current_transactions_total.clone()),
            Box::new(self.mempool_submission_added_count.clone()),
            Box::new(self.mempool_submission_rejected_count.clone()),
        ];

        for metric in metrics.into_iter() {
            registry.register(metric).unwrap();
        }
    }
}

// TODO - capture the metric types on a generic wrapper around the GenericCounter, and ensure the provided labels match the types, like in Java.
pub trait TakesMetricLabels {
    type Metric;

    fn with_label(&self, label1: impl MetricLabel) -> Self::Metric;
    fn with_two_labels(&self, label1: impl MetricLabel, label2: impl MetricLabel) -> Self::Metric;
    fn with_three_labels(
        &self,
        label1: impl MetricLabel,
        label2: impl MetricLabel,
        label3: impl MetricLabel,
    ) -> Self::Metric;
}

impl<T: MetricVecBuilder> TakesMetricLabels for MetricVec<T> {
    type Metric = <T as MetricVecBuilder>::M;

    fn with_label(&self, label1: impl MetricLabel) -> Self::Metric {
        self.with_label_values(&[label1.prometheus_label_name().as_ref()])
    }

    fn with_two_labels(&self, label1: impl MetricLabel, label2: impl MetricLabel) -> Self::Metric {
        self.with_label_values(&[
            label1.prometheus_label_name().as_ref(),
            label2.prometheus_label_name().as_ref(),
        ])
    }

    fn with_three_labels(
        &self,
        label1: impl MetricLabel,
        label2: impl MetricLabel,
        label3: impl MetricLabel,
    ) -> Self::Metric {
        self.with_label_values(&[
            label1.prometheus_label_name().as_ref(),
            label2.prometheus_label_name().as_ref(),
            label3.prometheus_label_name().as_ref(),
        ])
    }
}

/// Typically applied to enums or Errors where we wish to derive a label name.
/// Note the label name returned should be in a fixed, small-ish sized-set, to prevent
/// issues with tracking too many metrics, or combinatorial explosion of metrics with different labels.
pub trait MetricLabel {
    /// Typically &str, but could also be String if it's dynamic (as long as it's in a fixed small set)
    type StringReturnType: AsRef<str>;

    /// Returns the string label associated with this enum value.
    fn prometheus_label_name(&self) -> Self::StringReturnType;
}

/// We implement it for &T so that you can pass references in to function parameters taking `impl MetricLabel`
impl<T: MetricLabel> MetricLabel for &T {
    type StringReturnType = <T as MetricLabel>::StringReturnType;

    fn prometheus_label_name(&self) -> Self::StringReturnType {
        T::prometheus_label_name(self)
    }
}
