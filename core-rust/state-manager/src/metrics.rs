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

use std::time::{SystemTime, UNIX_EPOCH};

use crate::limits::{ExecutionMetrics, VertexLimitsExceeded};
use crate::transaction::{ConfigType, ExecutionConfigurator, LeaderRoundCounter};
use crate::StateVersion;
use node_common::config::limits::*;
use node_common::metrics::*;
use prometheus::*;

use radix_engine_common::prelude::*;

pub struct LedgerMetrics {
    address_encoder: AddressBech32Encoder, // for label rendering only
    pub state_version: IntGauge,
    pub transactions_committed: IntCounter,
    pub consensus_rounds_committed: IntCounterVec,
    pub last_update_epoch_second: Gauge,
    pub last_update_proposer_epoch_second: Gauge,
}

pub struct CommittedTransactionsMetrics {
    pub size: Histogram,
    pub execution_cost_units_consumed: Histogram,
    pub substate_read_size: Histogram,
    pub substate_read_count: Histogram,
    pub substate_write_size: Histogram,
    pub substate_write_count: Histogram,
    pub max_wasm_memory_used: Histogram,
    pub max_invoke_payload_size: Histogram,
}

pub struct VertexPrepareMetrics {
    pub proposal_transactions_size: Histogram,
    pub wasted_proposal_bandwidth: Histogram,
    pub stop_reason: IntCounterVec,
}

impl LedgerMetrics {
    pub fn new(network: &NetworkDefinition, registry: &Registry) -> Self {
        Self {
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
        }
    }

    pub fn update(
        &self,
        added_transactions: usize,
        new_state_version: StateVersion,
        validator_proposal_counters: Vec<(ComponentAddress, LeaderRoundCounter)>,
        proposer_timestamp_ms: i64,
    ) {
        self.state_version.set(new_state_version.number() as i64);
        self.transactions_committed
            .inc_by(added_transactions as u64);
        for (validator_address, counter) in validator_proposal_counters {
            let encoded_validator_address = self
                .address_encoder
                .encode(validator_address.as_ref())
                // a fallback for an unlikely encoding error:
                .unwrap_or_else(|_| validator_address.to_hex());
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
    }
}

pub struct TransactionMetricsData {
    size: usize,
    execution: ExecutionMetrics,
}

impl TransactionMetricsData {
    pub fn new(size: usize, execution: ExecutionMetrics) -> Self {
        TransactionMetricsData { size, execution }
    }
}

// TODO: update buckets limits when default values are overwritten
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
            substate_read_size: new_histogram(
                opts(
                    "committed_transactions_substate_read_size",
                    "Total (per committed transaction) substate read size in bytes.",
                ),
                // TODO(RC): update once max substate reads can be limited at execution
                higher_resolution_for_lower_values_buckets_for_limit(
                    DEFAULT_MAX_TOTAL_VERTEX_SUBSTATE_READ_SIZE,
                ),
            )
            .registered_at(registry),
            substate_read_count: new_histogram(
                opts(
                    "committed_transactions_substate_read_count",
                    "Number of substate reads per committed transactions.",
                ),
                higher_resolution_for_lower_values_buckets_for_limit(
                    DEFAULT_MAX_TOTAL_VERTEX_SUBSTATE_READ_COUNT,
                ),
            )
            .registered_at(registry),
            substate_write_size: new_histogram(
                opts(
                    "committed_transactions_substate_write_size",
                    "Total (per committed transaction) substate write size in bytes.",
                ),
                // TODO(RCnet-V3): update once max substate writes can be limited at execution
                higher_resolution_for_lower_values_buckets_for_limit(
                    DEFAULT_MAX_TOTAL_VERTEX_SUBSTATE_WRITE_SIZE,
                ),
            )
            .registered_at(registry),
            substate_write_count: new_histogram(
                opts(
                    "committed_transactions_substate_write_count",
                    "Number of substate writes per committed transactions.",
                ),
                // TODO(RCnet-V3): update once max substate writes can be limited at execution
                higher_resolution_for_lower_values_buckets_for_limit(
                    DEFAULT_MAX_TOTAL_VERTEX_SUBSTATE_WRITE_COUNT,
                ),
            )
            .registered_at(registry),
            max_wasm_memory_used: new_histogram(
                opts(
                    "committed_transactions_max_wasm_memory_used",
                    "Maximum WASM memory used in bytes per committed transaction.",
                ),
                // TODO(RCnet-V3): Just a placeholder until we figure out ExecutionMetrics.
                higher_resolution_for_lower_values_buckets_for_limit(10 * 1024 * 1024),
            )
            .registered_at(registry),
            max_invoke_payload_size: new_histogram(
                opts(
                    "committed_transactions_max_invoke_payload_size",
                    "Maximum invoke payload size in bytes per committed transaction.",
                ),
                higher_resolution_for_lower_values_buckets_for_limit(
                    execution_configurator
                        .execution_configs
                        .get(&ConfigType::Regular)
                        .unwrap()
                        .max_invoke_input_size,
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
                    .execution
                    .execution_cost_units_consumed as f64,
            );
            self.substate_read_size
                .observe(transaction_metrics_data.execution.substate_read_size as f64);
            self.substate_read_count
                .observe(transaction_metrics_data.execution.substate_read_count as f64);
            self.substate_write_size
                .observe(transaction_metrics_data.execution.substate_write_size as f64);
            self.substate_write_count
                .observe(transaction_metrics_data.execution.substate_write_count as f64);
            self.max_wasm_memory_used
                .observe(transaction_metrics_data.execution.max_wasm_memory_used as f64);
            self.max_invoke_payload_size
                .observe(transaction_metrics_data.execution.max_invoke_payload_size as f64);
        }
    }
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
    LimitExceeded(VertexLimitsExceeded),
}

impl MetricLabel for VertexPrepareStopReason {
    type StringReturnType = &'static str;

    fn prometheus_label_name(&self) -> Self::StringReturnType {
        match self {
            VertexPrepareStopReason::ProposalComplete => "ProposalComplete",
            VertexPrepareStopReason::EpochChange => "EpochChange",
            VertexPrepareStopReason::LimitExceeded(limit_exceeded) => match limit_exceeded {
                VertexLimitsExceeded::TransactionsCount => "TransactionsCountLimitReached",
                VertexLimitsExceeded::TransactionsSize => "TransactionsSizeLimitReached",
                VertexLimitsExceeded::ExecutionCostUnitsConsumed => {
                    "ExecutionCostUnitsConsumedLimitReached"
                }
                VertexLimitsExceeded::SubstateReadSize => "SubstateReadSizeLimitReached",
                VertexLimitsExceeded::SubstateReadCount => "SubstateReadCountLimitReached",
                VertexLimitsExceeded::SubstateWriteSize => "SubstateWriteSizeLimitReached",
                VertexLimitsExceeded::SubstateWriteCount => "SubstateWriteCountLimitReached",
            },
        }
    }
}
