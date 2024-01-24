/* Copyright 2021 Radix Publishing Ltd incorporated in Jersey (Channel Islands).
 *
 * Licensed under the Radix License, Version 1.0 (the "License"); you may not use this
 * file except in compliance with the License. You may obtain a copy of the License at:
 *
 * radixfoundation.org/licenses/LICENSE-v1
 *
 * The Licensor hereby grants permission for the Canonical version of the Work to be
 * published, distributed and used under or by reference to the Licensor’s trademark
 * Radix ® and use of any unregistered trade names, logos or get-up.
 *
 * The Licensor provides the Work (and each Contributor provides its Contributions) on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied,
 * including, without limitation, any warranties or conditions of TITLE, NON-INFRINGEMENT,
 * MERCHANTABILITY, or FITNESS FOR A PARTICULAR PURPOSE.
 *
 * Whilst the Work is capable of being deployed, used and adopted (instantiated) to create
 * a distributed ledger it is your responsibility to test and validate the code, together
 * with all logic and performance of that code under all foreseeable scenarios.
 *
 * The Licensor does not make or purport to make and hereby excludes liability for all
 * and any representation, warranty or undertaking in any form whatsoever, whether express
 * or implied, to any entity or person, including any representation, warranty or
 * undertaking, as to the functionality security use, value or other characteristics of
 * any distributed ledger nor in respect the functioning or value of any tokens which may
 * be created stored or transferred using the Work. The Licensor does not warrant that the
 * Work or any use of the Work complies with any law or regulation in any territory where
 * it may be implemented or used or that it will be appropriate for any specific purpose.
 *
 * Neither the licensor nor any current or former employees, officers, directors, partners,
 * trustees, representatives, agents, advisors, contractors, or volunteers of the Licensor
 * shall be liable for any direct or indirect, special, incidental, consequential or other
 * losses of any kind, in tort, contract or otherwise (including but not limited to loss
 * of revenue, income or profits, or loss of use or data, or loss of reputation, or loss
 * of any economic or other opportunity of whatsoever nature or howsoever arising), arising
 * out of or in connection with (without limitation of any use, misuse, of any ledger system
 * or use made or its functionality or any performance or operation of any code or protocol
 * caused by bugs or programming or logic errors or otherwise);
 *
 * A. any offer, purchase, holding, use, sale, exchange or transmission of any
 * cryptographic keys, tokens or assets created, exchanged, stored or arising from any
 * interaction with the Work;
 *
 * B. any failure in a transmission or loss of any token or assets keys or other digital
 * artefacts due to errors in transmission;
 *
 * C. bugs, hacks, logic errors or faults in the Work or any communication;
 *
 * D. system software or apparatus including but not limited to losses caused by errors
 * in holding or transmitting tokens by any third-party;
 *
 * E. breaches or failure of security including hacker attacks, loss or disclosure of
 * password, loss of private key, unauthorised use or misuse of such passwords or keys;
 *
 * F. any losses including loss of anticipated savings or other benefits resulting from
 * use of the Work or any changes to the Work (however implemented).
 *
 * You are solely responsible for; testing, validating and evaluation of all operation
 * logic, functionality, security and appropriateness of using the Work for any commercial
 * or non-commercial purpose and for any reproduction or redistribution by You of the
 * Work. You assume all risks associated with Your use of the Work and the exercise of
 * permissions under this License.
 */

use std::cmp::min;

use std::time::{SystemTime, UNIX_EPOCH};

use crate::limits::VertexLimitsExceeded;
use crate::transaction::{ExecutionConfigurator, LeaderRoundCounter};
use crate::{StateVersion, ValidatorId};
use node_common::config::limits::*;
use node_common::locks::{LockFactory, Mutex};
use node_common::metrics::*;
use prometheus::{
    Gauge, GaugeVec, Histogram, IntCounter, IntCounterVec, IntGauge, IntGaugeVec, Opts, Registry,
};
use radix_engine::blueprints::consensus_manager::EpochChangeEvent;

use crate::protocol::{
    PendingProtocolUpdateState, ProtocolState, ProtocolUpdateEnactmentCondition,
};
use crate::store::traits::measurement::CategoryDbVolumeStatistic;
use radix_engine::transaction::TransactionFeeSummary;
use radix_engine_common::prelude::*;

pub struct LedgerMetrics {
    address_encoder: AddressBech32Encoder, // for label rendering only
    pub state_version: IntGauge,
    pub transactions_committed: IntCounter,
    pub consensus_rounds_committed: IntCounterVec,
    pub self_consensus_rounds_committed: IntCounterVec, // a subset of the above, for convenience
    pub last_update_epoch_second: Gauge,
    pub last_update_proposer_epoch_second: Gauge,
    pub recent_self_proposal_miss_count: ValidatorProposalMissTracker,
    pub recent_proposer_timestamp_progress_rate: ProposerTimestampProgressRateTracker,
}

pub struct CommittedTransactionsMetrics {
    pub size: Histogram,
    pub execution_cost_units_consumed: Histogram,
    pub finalization_cost_units_consumed: Histogram,
}

pub struct ProtocolMetrics {
    pub protocol_update_readiness_ratio: GaugeVec,
    pub pending_update_threshold_required_ratio: GaugeVec,
    pub pending_update_threshold_required_consecutive_epochs: IntGaugeVec,
    pub pending_update_threshold_current_consecutive_epochs: IntGaugeVec,
    pub pending_update_lower_bound_epoch: IntGaugeVec,
    pub pending_update_upper_bound_epoch: IntGaugeVec,
    pub enacted_protocol_update_state_version: IntGaugeVec,
}

pub struct VertexPrepareMetrics {
    pub proposal_transactions_size: Histogram,
    pub wasted_proposal_bandwidth: Histogram,
    pub stop_reason: IntCounterVec,
}

pub struct RawDbMetrics {
    pub uncompacted_live_entries: IntGaugeVec,
    pub uncompacted_tombstone_entries: IntGaugeVec,
    pub size: IntGaugeVec,
    pub files: IntGaugeVec,
    pub max_level: IntGaugeVec,
}

impl LedgerMetrics {
    pub fn new(
        network: &NetworkDefinition,
        lock_factory: LockFactory,
        registry: &Registry,
        current_ledger_proposer_timestamp_ms: i64,
    ) -> Self {
        let instance = Self {
            address_encoder: AddressBech32Encoder::new(network),
            state_version: IntGauge::with_opts(opts(
                "ledger_state_version",
                "Version of the ledger state.",
            ))
            .registered_at(registry),
            transactions_committed: IntCounter::with_opts(opts(
                "ledger_transactions_committed_total",
                "Count of transactions committed to the ledger.",
            ))
            .registered_at(registry),
            consensus_rounds_committed: IntCounterVec::new(
                opts(
                    "ledger_consensus_rounds_committed",
                    "Count of rounds processed by consensus that reached the ledger commit.",
                ),
                &["leader_component_address", "round_resolution"],
            )
            .registered_at(registry),
            self_consensus_rounds_committed: IntCounterVec::new(
                opts(
                    "ledger_self_consensus_rounds_committed",
                    "Count of rounds lead by this validator that reached the ledger commit.",
                ),
                &["round_resolution"],
            )
            .registered_at(registry),
            last_update_epoch_second: Gauge::with_opts(opts(
                "ledger_last_update_epoch_second",
                "Last timestamp at which the ledger was updated.",
            ))
            .registered_at(registry),
            last_update_proposer_epoch_second: Gauge::with_opts(opts(
                "ledger_last_update_proposer_epoch_second",
                "Proposer timestamp from the last proof written to the ledger.",
            ))
            .registered_at(registry),
            recent_self_proposal_miss_count: ValidatorProposalMissTracker::new(
                opts(
                    "ledger_recent_self_proposal_miss_count",
                    &format!("A number of proposals missed by this validator during its {} most recent rounds.", PROPOSAL_HISTORY_LEN),
                ),
                lock_factory.named("self_proposal_miss_tracker"),
                registry,
            ),
            recent_proposer_timestamp_progress_rate: ProposerTimestampProgressRateTracker::new(
                current_ledger_proposer_timestamp_ms,
                opts(
                    "ledger_recent_proposer_timestamp_progress_rate",
                    &format!("A rate of the proposer timestamp progress (against wall-clock) averaged over {} most recent ledger updates.", PROGRESS_RATE_HISTORY_LEN),
                ),
                lock_factory.named("progress_rate_tracker"),
                registry,
            ),
        };
        OverallLedgerHealthFactor::register_direct_collector(
            &instance,
            opts(
                "ledger_overall_health_factor",
                "A proper fraction representing an overall local ledger health (with 0.0 = critical and 1.0 = healthy).",
            ),
            registry
        );
        instance
    }

    pub fn update(
        &self,
        added_transactions: usize,
        new_state_version: StateVersion,
        validator_proposal_counters: Vec<(ValidatorId, LeaderRoundCounter)>,
        proposer_timestamp_ms: i64,
        self_validator_id: Option<ValidatorId>,
    ) {
        self.state_version.set(new_state_version.number() as i64);
        self.transactions_committed
            .inc_by(added_transactions as u64);
        for (validator_id, counter) in validator_proposal_counters {
            let is_self = self_validator_id == Some(validator_id);
            let encoded_validator_address = self
                .address_encoder
                .encode(validator_id.component_address.as_ref())
                // a fallback for an unlikely encoding error:
                .unwrap_or_else(|_| validator_id.component_address.to_hex());
            for (round_resolution, count) in [
                (ConsensusRoundResolution::Successful, counter.successful),
                (
                    ConsensusRoundResolution::MissedByFallback,
                    counter.missed_by_fallback,
                ),
                (ConsensusRoundResolution::MissedByGap, counter.missed_by_gap),
            ] {
                self.consensus_rounds_committed
                    .with_two_labels(&encoded_validator_address, round_resolution)
                    .inc_by(count as u64);
                if is_self {
                    self.self_consensus_rounds_committed
                        .with_label(round_resolution)
                        .inc_by(count as u64);
                }
            }
            if is_self {
                self.recent_self_proposal_miss_count.track(&counter);
            }
        }
        self.last_update_epoch_second.set(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
        );
        self.last_update_proposer_epoch_second
            .set(proposer_timestamp_ms as f64 / 1000.0);
        self.recent_proposer_timestamp_progress_rate
            .track(proposer_timestamp_ms);
    }

    /// Calculates current [`LedgerStatus`] (see the enum's doc for explanation).
    pub fn get_ledger_status(&self) -> LedgerStatus {
        if current_wallclock_epoch_sec() - self.last_update_proposer_epoch_second.get()
            < SYNCED_LEDGER_MAX_DELAY_SEC
        {
            LedgerStatus::Synced
        } else if self.recent_proposer_timestamp_progress_rate.gauge.get()
            > MIN_PROPOSER_TIMESTAMP_PROGRESS_RATE
        {
            LedgerStatus::Syncing
        } else {
            LedgerStatus::NotSyncing
        }
    }

    /// Returns the current value of [`recent_self_proposal_miss_count`] and the tracked history
    /// length (for context).
    pub fn get_recent_self_proposal_miss_statistic(&self) -> RecentSelfProposalMissStatistic {
        RecentSelfProposalMissStatistic {
            missed_count: u64::try_from(self.recent_self_proposal_miss_count.gauge.get())
                .expect("negative count"),
            recent_proposals_tracked_count: u64::try_from(PROPOSAL_HISTORY_LEN)
                .expect("negative history length"),
        }
    }
}

pub struct TransactionMetricsData {
    size: usize,
    fee_summary: TransactionFeeSummary,
}

impl TransactionMetricsData {
    pub fn new(size: usize, fee_summary: TransactionFeeSummary) -> Self {
        TransactionMetricsData { size, fee_summary }
    }
}

impl CommittedTransactionsMetrics {
    pub fn new(registry: &Registry, execution_configurator: &ExecutionConfigurator) -> Self {
        Self {
            size: new_histogram(
                opts(
                    "committed_transactions_size",
                    "Size in bytes of committed transactions.",
                ),
                higher_resolution_for_lower_values_buckets_for_limit(MAX_TRANSACTION_SIZE),
            )
            .registered_at(registry),
            execution_cost_units_consumed: new_histogram(
                opts(
                    "committed_transactions_execution_cost_units_consumed",
                    "Execution cost units consumed per committed transactions.",
                ),
                higher_resolution_for_lower_values_buckets_for_limit(
                    execution_configurator
                        .costing_parameters
                        .execution_cost_unit_limit as usize,
                ),
            )
            .registered_at(registry),
            finalization_cost_units_consumed: new_histogram(
                opts(
                    "committed_transactions_finalization_cost_units_consumed",
                    "Finalization cost units consumed per committed transactions.",
                ),
                higher_resolution_for_lower_values_buckets_for_limit(
                    execution_configurator
                        .costing_parameters
                        .finalization_cost_unit_limit as usize,
                ),
            )
            .registered_at(registry),
        }
    }

    pub fn update(&self, transactions_metrics_data: Vec<TransactionMetricsData>) {
        for transaction_metrics_data in transactions_metrics_data {
            self.size.observe(transaction_metrics_data.size as f64);
            self.execution_cost_units_consumed.observe(
                transaction_metrics_data
                    .fee_summary
                    .total_execution_cost_units_consumed as f64,
            );
            self.finalization_cost_units_consumed.observe(
                transaction_metrics_data
                    .fee_summary
                    .total_finalization_cost_units_consumed as f64,
            );
        }
    }
}

impl ProtocolMetrics {
    pub fn new(registry: &Registry, initial_protocol_state: &ProtocolState) -> Self {
        let instance = Self {
            protocol_update_readiness_ratio: GaugeVec::new(
                opts(
                    "protocol_update_readiness_ratio",
                    "A ratio of supporting stake to total stake in the current validator set.",
                ),
                &["readiness_signal_name"],
            )
            .registered_at(registry),
            pending_update_threshold_required_ratio: GaugeVec::new(
                opts(
                    "protocol_update_pending_threshold_required_ratio",
                    "Required readiness ratio for the given protocol update threshold.",
                ),
                &["protocol_version_name", "readiness_signal_name", "threshold_index"],
            )
                .registered_at(registry),
            pending_update_threshold_required_consecutive_epochs: IntGaugeVec::new(
                opts(
                    "protocol_update_pending_threshold_required_consecutive_epochs",
                    "Required number of consecutive epochs of support for the given protocol update threshold.",
                ),
                &["protocol_version_name", "readiness_signal_name", "threshold_index"],
            )
                .registered_at(registry),
            pending_update_threshold_current_consecutive_epochs: IntGaugeVec::new(
                opts(
                    "protocol_update_pending_threshold_current_consecutive_epochs",
                    "Current number of consecutive epochs of support for the given protocol update threshold.",
                ),
                &["protocol_version_name", "readiness_signal_name", "threshold_index"],
            )
                .registered_at(registry),
            pending_update_lower_bound_epoch: IntGaugeVec::new(
                opts(
                    "protocol_update_pending_lower_bound_epoch",
                    "Earliest epoch when the given protocol update can be enacted (inclusive)",
                ),
                &["protocol_version_name", "readiness_signal_name"],
            )
                .registered_at(registry),
            pending_update_upper_bound_epoch: IntGaugeVec::new(
                opts(
                    "protocol_update_pending_upper_bound_epoch",
                    "Upper bound epoch for the given protocol update (exclusive)",
                ),
                &["protocol_version_name", "readiness_signal_name"],
            )
                .registered_at(registry),
            enacted_protocol_update_state_version: IntGaugeVec::new(
                opts(
                    "protocol_update_enacted_state_version",
                    "State version at which the protocol update was enacted (init proof)",
                ),
                &["protocol_version_name"],
            )
                .registered_at(registry),
        };

        instance.update_state_based_metrics(initial_protocol_state);

        instance
    }

    pub fn update(&self, protocol_state: &ProtocolState, epoch_change: &EpochChangeEvent) {
        self.update_state_based_metrics(protocol_state);
        self.update_epoch_change_based_metrics(epoch_change);
    }

    /// Updates the metrics that are based on ProtocolState (pending, enacted updates)
    fn update_state_based_metrics(&self, protocol_state: &ProtocolState) {
        // Reset the metrics (to clear leftover pending updates as they transition to enacted)
        self.pending_update_threshold_required_ratio.reset();
        self.pending_update_threshold_required_consecutive_epochs
            .reset();
        self.pending_update_threshold_current_consecutive_epochs
            .reset();
        self.pending_update_lower_bound_epoch.reset();
        self.pending_update_upper_bound_epoch.reset();
        self.enacted_protocol_update_state_version.reset();

        for pending_protocol_update in protocol_state.pending_protocol_updates.iter() {
            let protocol_update = &pending_protocol_update.protocol_update;
            match &pending_protocol_update.protocol_update.enactment_condition {
                ProtocolUpdateEnactmentCondition::EnactAtStartOfEpochIfValidatorsReady {
                    lower_bound_inclusive,
                    upper_bound_exclusive,
                    ..
                } => {
                    let readiness_signal_name = protocol_update.readiness_signal_name();
                    self.pending_update_lower_bound_epoch
                        .with_two_labels(
                            protocol_update.next_protocol_version.to_string(),
                            readiness_signal_name.to_string(),
                        )
                        .set(lower_bound_inclusive.number() as i64);
                    self.pending_update_upper_bound_epoch
                        .with_two_labels(
                            protocol_update.next_protocol_version.to_string(),
                            readiness_signal_name.to_string(),
                        )
                        .set(upper_bound_exclusive.number() as i64);
                    match &pending_protocol_update.state {
                        PendingProtocolUpdateState::ForSignalledReadinessSupportCondition {
                            thresholds_state,
                        } => {
                            for (index, (threshold, threshold_state)) in
                                thresholds_state.iter().enumerate()
                            {
                                self.pending_update_threshold_required_ratio
                                    .with_three_labels(
                                        protocol_update.next_protocol_version.to_string(),
                                        readiness_signal_name.to_string(),
                                        index.to_string(),
                                    )
                                    .set(dec_to_f64_for_metrics(
                                        &threshold.required_ratio_of_stake_supported,
                                    ));
                                self.pending_update_threshold_required_consecutive_epochs
                                    .with_three_labels(
                                        protocol_update.next_protocol_version.to_string(),
                                        readiness_signal_name.to_string(),
                                        index.to_string(),
                                    )
                                    .set(
                                        threshold.required_consecutive_completed_epochs_of_support
                                            as i64,
                                    );
                                self.pending_update_threshold_current_consecutive_epochs
                                    .with_three_labels(
                                        protocol_update.next_protocol_version.to_string(),
                                        readiness_signal_name.to_string(),
                                        index.to_string(),
                                    )
                                    .set(
                                        threshold_state.consecutive_started_epochs_of_support
                                            as i64,
                                    );
                            }
                        }
                        PendingProtocolUpdateState::Empty => { /* no-op, shouldn't happen */ }
                    }
                }
                ProtocolUpdateEnactmentCondition::EnactAtStartOfEpochUnconditionally(epoch) => {
                    self.pending_update_lower_bound_epoch
                        .with_two_labels(
                            protocol_update.next_protocol_version.to_string(),
                            "".to_string(),
                        )
                        .set(epoch.number() as i64);
                    self.pending_update_upper_bound_epoch
                        .with_two_labels(
                            protocol_update.next_protocol_version.to_string(),
                            "".to_string(),
                        )
                        .set(epoch.number() as i64 + 1);
                }
            }
        }

        for (state_version, protocol_version_name) in protocol_state.enacted_protocol_updates.iter()
        {
            self.enacted_protocol_update_state_version
                .with_label(protocol_version_name.to_string())
                .set(state_version.number() as i64);
        }
    }

    /// Updates the metrics that are based on epoch change event
    fn update_epoch_change_based_metrics(&self, epoch_change: &EpochChangeEvent) {
        self.protocol_update_readiness_ratio.reset();

        let total_stake = epoch_change
            .validator_set
            .total_active_stake_xrd()
            .expect("Failed to calculate the total stake");
        for (readiness_signal_name, stake_readiness) in
            epoch_change.significant_protocol_update_readiness.iter()
        {
            let readiness_ratio = stake_readiness
                .checked_div(total_stake)
                .unwrap_or(Decimal::ZERO);
            self.protocol_update_readiness_ratio
                .with_label(readiness_signal_name)
                .set(dec_to_f64_for_metrics(&readiness_ratio))
        }
    }
}

/// Unsafe, metrics-only conversion
fn dec_to_f64_for_metrics(input: &Decimal) -> f64 {
    f64::from_str(&input.to_string()).unwrap_or(0f64)
}

impl VertexPrepareMetrics {
    pub fn new(registry: &Registry) -> Self {
        Self {
            proposal_transactions_size: new_histogram(
                opts(
                    "vertex_prepare_proposal_transactions_size",
                    "Size of all transactions inside proposal.",
                ),
                // TODO: This is accurate enough but update once MAX_PROPOSAL_SIZE is available here
                higher_resolution_for_higher_values_buckets_for_limit(
                    DEFAULT_MAX_TOTAL_VERTEX_TRANSACTIONS_SIZE,
                ),
            )
            .registered_at(registry),
            wasted_proposal_bandwidth: new_histogram(
                opts(
                    "vertex_prepare_wasted_proposal_bandwidth",
                    "Size sum of received transactions that were skipped during preparation.",
                ),
                // TODO: This is accurate enough but update once MAX_PROPOSAL_SIZE is available here
                higher_resolution_for_lower_values_buckets_for_limit(
                    DEFAULT_MAX_TOTAL_VERTEX_TRANSACTIONS_SIZE,
                ),
            )
            .registered_at(registry),
            stop_reason: IntCounterVec::new(
                opts(
                    "vertex_prepare_stop_reason",
                    "Count of vertex prepare stop reasons by type.",
                ),
                &["type"],
            )
            .registered_at(registry),
        }
    }

    pub fn update(
        &self,
        total_proposal_size: usize,
        committed_proposal_size: usize,
        stop_reason: VertexPrepareStopReason,
    ) {
        self.proposal_transactions_size
            .observe(total_proposal_size as f64);
        self.wasted_proposal_bandwidth
            .observe((total_proposal_size - committed_proposal_size) as f64);
        self.stop_reason.with_label(stop_reason).inc();
    }
}

impl RawDbMetrics {
    pub fn new(registry: &Registry) -> Self {
        Self {
            uncompacted_live_entries: IntGaugeVec::new(
                opts(
                    "raw_db_uncompacted_live_entries",
                    "A sum of live entry counts across SST files, by category.",
                ),
                &["category"],
            )
            .registered_at(registry),
            uncompacted_tombstone_entries: IntGaugeVec::new(
                opts(
                    "raw_db_uncompacted_tombstone_entries",
                    "A sum of tombstone entry counts across SST files, by category.",
                ),
                &["category"],
            )
            .registered_at(registry),
            size: IntGaugeVec::new(
                opts(
                    "raw_db_size",
                    "A sum of all SST file sizes holding a specific category, in bytes.",
                ),
                &["category"],
            )
            .registered_at(registry),
            files: IntGaugeVec::new(
                opts(
                    "raw_db_files",
                    "A number of SST files holding a specific category, in bytes.",
                ),
                &["category"],
            )
            .registered_at(registry),
            max_level: IntGaugeVec::new(
                opts(
                    "raw_db_max_level",
                    "A maximum level of an SST file, by category",
                ),
                &["category"],
            )
            .registered_at(registry),
        }
    }

    pub fn update(&self, statistics: impl IntoIterator<Item = CategoryDbVolumeStatistic>) {
        for statistic in statistics {
            self.uncompacted_live_entries
                .with_label(&statistic.category_name)
                .set(i64::try_from(statistic.live_count).unwrap_or_default());
            self.uncompacted_tombstone_entries
                .with_label(&statistic.category_name)
                .set(i64::try_from(statistic.tombstone_count).unwrap_or_default());
            self.size
                .with_label(&statistic.category_name)
                .set(i64::try_from(statistic.size_bytes).unwrap_or_default());
            self.files
                .with_label(&statistic.category_name)
                .set(i64::try_from(statistic.sst_count).unwrap_or_default());
            self.max_level
                .with_label(&statistic.category_name)
                .set(i64::from(statistic.max_level));
        }
    }
}

// Concrete types used for metric labels:
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsensusRoundResolution {
    Successful,
    MissedByFallback,
    MissedByGap,
}

impl MetricLabel for ConsensusRoundResolution {
    type StringReturnType = &'static str;

    fn prometheus_label_name(&self) -> Self::StringReturnType {
        match *self {
            ConsensusRoundResolution::Successful => "Successful",
            ConsensusRoundResolution::MissedByFallback => "MissedByFallback",
            ConsensusRoundResolution::MissedByGap => "MissedByGap",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VertexPrepareStopReason {
    ProposalComplete,
    EpochChange,
    ProtocolUpdate,
    LimitExceeded(VertexLimitsExceeded),
}

impl MetricLabel for VertexPrepareStopReason {
    type StringReturnType = &'static str;

    fn prometheus_label_name(&self) -> Self::StringReturnType {
        match self {
            VertexPrepareStopReason::ProposalComplete => "ProposalComplete",
            VertexPrepareStopReason::EpochChange => "EpochChange",
            VertexPrepareStopReason::ProtocolUpdate => "ProtocolUpdate",
            VertexPrepareStopReason::LimitExceeded(limit_exceeded) => match limit_exceeded {
                VertexLimitsExceeded::TransactionsCount => "TransactionsCountLimitReached",
                VertexLimitsExceeded::TransactionsSize => "TransactionsSizeLimitReached",
                VertexLimitsExceeded::ExecutionCostUnitsConsumed => {
                    "ExecutionCostUnitsConsumedLimitReached"
                }
                VertexLimitsExceeded::FinalizationCostUnitsConsumed => {
                    "FinalizationCostUnitsConsumedLimitReached"
                }
            },
        }
    }
}

/// A number of most recent [`RoundSlot`]s of a single validator to track for metrics purposes.
const PROPOSAL_HISTORY_LEN: usize = 100;

/// An indication of (any kind of) missed round vs successful round of a validator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RoundSlot {
    Success,
    Missed,
}

/// A higher-level metric helper, tracking a number of recent proposal misses of a specific
/// validator (i.e. validator whose LeaderRoundCounter is passed to `track`).
pub struct ValidatorProposalMissTracker {
    buffer: Mutex<RingBuffer<RoundSlot, PROPOSAL_HISTORY_LEN>>,
    gauge: IntGauge,
}

impl ValidatorProposalMissTracker {
    /// Creates a new tracker and registers its resulting [`IntGauge`] (with the given options) at
    /// the given registry.
    /// Note: the [`LockFactory`] is required to ensure a thread-safe access to a ring-buffer used
    /// for history tracking.
    pub fn new(opts: Opts, lock_factory: LockFactory, registry: &Registry) -> Self {
        Self {
            buffer: lock_factory.new_mutex(RingBuffer::new(RoundSlot::Success)),
            gauge: IntGauge::with_opts(opts).registered_at(registry),
        }
    }

    /// Interprets the newest round history of the scoped validator and updates the managed gauge of
    /// recent proposal misses.
    pub fn track(&self, counter: &LeaderRoundCounter) {
        // Optimization: even if lots of rounds were missed, we track only the "recent" number.
        let new_missed_count = min(
            counter.missed(),
            PROPOSAL_HISTORY_LEN + (counter.missed() % PROPOSAL_HISTORY_LEN),
        ) as i64;
        // We are not actually getting a time-ordered history - only a statistic. We have to invent
        // the order, so we put the successes first...
        let mut buffer = self.buffer.lock();
        let mut outdated_missed_count = 0;
        for _ in 0..counter.successful {
            let outdated = buffer.put(RoundSlot::Success);
            if outdated == RoundSlot::Missed {
                outdated_missed_count += 1;
            }
        }
        // ... and the misses later (so that they stay in the buffer longer).
        for _ in 0..new_missed_count {
            let outdated = buffer.put(RoundSlot::Missed);
            if outdated == RoundSlot::Missed {
                outdated_missed_count += 1;
            }
        }
        // We update the gauge with a delta of new observed misses vs those that ceased to be recent
        self.gauge.add(new_missed_count - outdated_missed_count);
    }
}

/// A number of most recent [`ProposerTimestampDatapoint`]s to track for metrics purposes.
/// We mostly care about the progress rate during ledger-syncing, where Node is capable of ingesting
/// >100 sync responses a second. The value below should give us at least 1 sec of history.
const PROGRESS_RATE_HISTORY_LEN: usize = 100;

/// A `proposer_timestamp_ms` captured at a specific `wallclock_epoch_sec`.
#[derive(Debug, Clone, Copy)]
struct ProposerTimestampDatapoint {
    wallclock_epoch_sec: f64,
    proposer_timestamp_ms: i64,
}

impl ProposerTimestampDatapoint {
    /// Captures the given `proposer_timestamp_ms` at the current wall-clock.
    pub fn at_current_wallclock(proposer_timestamp_ms: i64) -> Self {
        Self {
            wallclock_epoch_sec: current_wallclock_epoch_sec(),
            proposer_timestamp_ms,
        }
    }

    /// Calculates the rate of the `proposer_timestamp_ms` measured relative to the given reference
    /// point.
    pub fn proposer_timestamp_rate_since(&self, reference: &ProposerTimestampDatapoint) -> f64 {
        let delta_proposer_timestamp_ms =
            self.proposer_timestamp_ms - reference.proposer_timestamp_ms;
        let delta_proposer_timestamp_sec = (delta_proposer_timestamp_ms as f64) / 1000.0;
        let delta_wallclock_sec = self.wallclock_epoch_sec - reference.wallclock_epoch_sec;
        delta_proposer_timestamp_sec / delta_wallclock_sec
    }
}

/// A higher-level metric helper, tracking a recent rate of `proposer_timestamp_ms` committed to the
/// ledger.
pub struct ProposerTimestampProgressRateTracker {
    buffer: Mutex<RingBuffer<ProposerTimestampDatapoint, PROGRESS_RATE_HISTORY_LEN>>,
    gauge: Gauge,
}

impl ProposerTimestampProgressRateTracker {
    /// Creates a new tracker and registers its resulting [`Gauge`] (with the given options) at the
    /// given registry.
    /// Note: the [`LockFactory`] is required to ensure a thread-safe access to a ring-buffer used
    /// for history tracking.
    pub fn new(
        initial_proposer_timestamp_ms: i64,
        opts: Opts,
        lock_factory: LockFactory,
        registry: &Registry,
    ) -> Self {
        Self {
            buffer: lock_factory.new_mutex(RingBuffer::new(
                ProposerTimestampDatapoint::at_current_wallclock(initial_proposer_timestamp_ms),
            )),
            gauge: Gauge::with_opts(opts).registered_at(registry),
        }
    }

    /// Records the given currently-committed `proposer_timestamp_ms` and updates the managed gauge
    /// with its recent rate.
    pub fn track(&self, proposer_timestamp_ms: i64) {
        let mut buffer = self.buffer.lock();
        let current = ProposerTimestampDatapoint::at_current_wallclock(proposer_timestamp_ms);
        let outdated = buffer.put(current);
        let recent_rate = current.proposer_timestamp_rate_since(&outdated);
        self.gauge.set(recent_rate);
    }
}

/// A maximum delay of latest committed proposer timestamp relative to local wall-clock, in seconds,
/// up to which a Node considers itself "synced".
const SYNCED_LEDGER_MAX_DELAY_SEC: f64 = 60.0;

/// A minimum progress rate of the proposer timestamp (see `ProposerTimestampProgressRateTracker`),
/// below which a syncing Node is considered critically unhealthy.
const MIN_PROPOSER_TIMESTAMP_PROGRESS_RATE: f64 = 10.0;

/// A progress rate of the proposer timestamp (see `ProposerTimestampProgressRateTracker`) at or
/// above which a syncing Node is considered fully healthy.
const HEALTHY_PROPOSER_TIMESTAMP_PROGRESS_RATE: f64 = 50.0;

/// A number of recent proposal misses (see `ValidatorProposalMissTracker`) at or above which a Validator
/// is considered critically unhealthy.
/// Of course, missing no proposals is fully healthy. Missing some small number of proposals (i.e.
/// less than this constant) results in some proportionally lower health factor.
/// A Node outside Active Validator Set is considered fully healthy in this aspect (i.e. it misses
/// no proposals).
const CRITICAL_RECENT_PROPOSAL_MISS_COUNT: i64 = 2;

/// A top-level health metric helper, aggregating a few lower-level parts of [`LedgerMetrics`] into
/// one, single-dimensional "overall health factor" [`Gauge`].
struct OverallLedgerHealthFactor {
    last_update_proposer_epoch_second: Gauge,
    recent_proposer_timestamp_progress_rate: Gauge,
    recent_self_proposal_miss_count: IntGauge,
}

impl OverallLedgerHealthFactor {
    /// Creates a direct Prometheus collector (of `OverallLedgerHealthFactor::calculate()`) and
    /// registers it (with the given options) at the given registry.
    pub fn register_direct_collector(
        ledger_metrics: &LedgerMetrics,
        opts: Opts,
        registry: &Registry,
    ) {
        let health_factor = Self {
            last_update_proposer_epoch_second: ledger_metrics
                .last_update_proposer_epoch_second
                .clone(),
            recent_proposer_timestamp_progress_rate: ledger_metrics
                .recent_proposer_timestamp_progress_rate
                .gauge
                .clone(),
            recent_self_proposal_miss_count: ledger_metrics
                .recent_self_proposal_miss_count
                .gauge
                .clone(),
        };
        let collector = GetterGauge::new(move || health_factor.calculate(), opts);
        registry.register(Box::new(collector.unwrap())).unwrap();
    }

    /// Calculates the current value of the overall ledger health factor.
    /// This is a proper fraction representation, where:
    /// - 0.0 means "critically unhealthy",
    /// - 1.0 means "fully healthy",
    /// - intermediate fractions mean some level of warning.
    /// Implementation wise, the result depends on the "syncing rate" and "proposal reliability".
    fn calculate(&self) -> f64 {
        self.syncing_factor() * self.proposal_reliability_factor()
    }

    /// Calculates the health factor part related to ledger-syncing.
    /// If the ledger is synced (i.e. the latest committed proposer timestamp is close to the
    /// wall-clock), then the result is "fully healthy" (i.e. 1.0).
    /// Otherwise, the health factor depends on the proposer timestamp's progress rate (see
    /// [`HEALTHY_PROPOSER_TIMESTAMP_PROGRESS_RATE`] and [`MIN_PROPOSER_TIMESTAMP_PROGRESS_RATE`]).
    fn syncing_factor(&self) -> f64 {
        let proposer_timestamp_delay_sec =
            current_wallclock_epoch_sec() - self.last_update_proposer_epoch_second.get();
        if proposer_timestamp_delay_sec < SYNCED_LEDGER_MAX_DELAY_SEC {
            return 1.0;
        }
        let clamped_proposer_timestamp_rate = clamp(
            MIN_PROPOSER_TIMESTAMP_PROGRESS_RATE,
            self.recent_proposer_timestamp_progress_rate.get(),
            HEALTHY_PROPOSER_TIMESTAMP_PROGRESS_RATE,
        );
        (clamped_proposer_timestamp_rate - MIN_PROPOSER_TIMESTAMP_PROGRESS_RATE)
            / (HEALTHY_PROPOSER_TIMESTAMP_PROGRESS_RATE - MIN_PROPOSER_TIMESTAMP_PROGRESS_RATE)
    }

    /// Calculates the health factor part related to proposal reliability.
    /// It is "fully healthy" (i.e. 1.0) when no proposals are missed (also in case this Node is not
    /// in active validator set). Otherwise, it linearly degrades with the number of recently missed
    /// proposals, potentially down to 0.0 (see [`CRITICAL_RECENT_PROPOSAL_MISS_COUNT`]).
    fn proposal_reliability_factor(&self) -> f64 {
        let clamped_recent_proposal_miss_count = min(
            self.recent_self_proposal_miss_count.get(),
            CRITICAL_RECENT_PROPOSAL_MISS_COUNT,
        );
        1.0 - (clamped_recent_proposal_miss_count as f64)
            / (CRITICAL_RECENT_PROPOSAL_MISS_COUNT as f64)
    }
}

/// A simplified "overall ledger health" (see [`OverallLedgerHealthFactor::syncing_factor()`]).
/// This enum is meant to be surfaced from a `/system/health` API.
#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum LedgerStatus {
    /// Ledger is fully synced, i.e. the last committed proposer timestamp is closer than
    /// [`SYNCED_LEDGER_MAX_DELAY_SEC`] to wallclock.
    Synced,
    /// Ledger's last proposer timestamp is far from wallclock, but progresses at least
    /// [`MIN_PROPOSER_TIMESTAMP_PROGRESS_RATE`] times faster than wallclock (i.e. catches up).
    Syncing,
    /// Ledger's last proposer timestamp is far from wallclock *and* progresses slower than
    /// expected from a [`Self::Syncing`] ledger.
    NotSyncing,
}

/// A recent statistic on a number of successful/missed proposals.
/// This information is meant to be surfaced from a `/system/health` API.
#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct RecentSelfProposalMissStatistic {
    /// A number of missed proposals among [`recent_proposals_tracked_count`] most recent ones.
    missed_count: u64,
    /// A configured length of proposal miss tracking history.
    recent_proposals_tracked_count: u64,
}

/// Returns the `SystemTime::now()` expressed as a fractional number of seconds since epoch.
fn current_wallclock_epoch_sec() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64()
}

/// A "clamp" implementation for not-strictly-[`Ord`] elements (for which a classic `min()/max()`
/// idiom cannot be used).
fn clamp<T: Copy + PartialOrd>(min: T, arg: T, max: T) -> T {
    if arg < min {
        min
    } else if arg > max {
        max
    } else {
        arg
    }
}
