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

use std::{sync::Arc, time::Duration};

use node_common::{locks::RwLock, metrics::*};
use prometheus::{IntGaugeVec, Registry};
use tokio::{runtime::Runtime, sync::oneshot, time::interval};

use super::{
    traits::measurement::{CategoryDbVolumeStatistic, MeasurableDatabase},
    StateManagerDatabase,
};

#[derive(Clone)]
pub struct RawDbMetrics {
    pub entries: IntGaugeVec,
    pub size: IntGaugeVec,
}

impl RawDbMetrics {
    pub fn new(registry: &Registry) -> Self {
        Self {
            entries: IntGaugeVec::new(
                opts(
                    "raw_db_entries",
                    "An approximate number of entries persisted in the database, by category.",
                ),
                &["category"],
            )
            .registered_at(registry),
            size: IntGaugeVec::new(
                opts(
                    "raw_db_size",
                    "An approximate size of the database, in bytes, by category of entries.",
                ),
                &["category"],
            )
            .registered_at(registry),
        }
    }

    pub fn update(&self, statistics: impl IntoIterator<Item = CategoryDbVolumeStatistic>) {
        for statistic in statistics {
            self.entries
                .with_label(&statistic.category_name)
                .set(i64::try_from(statistic.entry_count).unwrap_or_default());
            self.size
                .with_label(&statistic.category_name)
                .set(i64::try_from(statistic.size_bytes).unwrap_or_default());
        }
    }
}

/// An interval between time-intensive measurement of raw DB metrics.
/// Some of our raw DB metrics take ~a few milliseconds to collect. We cannot afford the overhead of
/// updating them every time they change (i.e. on every DB commit) and we also should not perform
/// this considerable I/O within the Prometheus' exposition servlet thread - hence, a periodic task
/// (which in practice still runs more often than Prometheus' scraping).
const RAW_DB_MEASUREMENT_INTERVAL: Duration = Duration::from_secs(10);

#[derive(Clone)]
pub struct RawDBMetricsReportThread {
    database: Arc<RwLock<StateManagerDatabase>>,
    raw_db_metrics: RawDbMetrics,
}

impl RawDBMetricsReportThread {
    pub fn new(database: Arc<RwLock<StateManagerDatabase>>, metric_registry: &Registry) -> Self {
        Self {
            database,
            raw_db_metrics: RawDbMetrics::new(metric_registry),
        }
    }

    /// Starts a background thread responsible for periodic raw DB metrics collection, and returns a
    /// handle that keeps it running.
    /// See [`RAW_DB_MEASUREMENT_INTERVAL`] for more details.
    pub fn start(&self, runtime: &Runtime, shutdown_signal: oneshot::Receiver<()>) {
        let context = self.clone();
        runtime.spawn(async move {
            let mut shutdown_signal = shutdown_signal;
            let mut interval = interval(RAW_DB_MEASUREMENT_INTERVAL);

            loop {
                tokio::select! {
                    _ = &mut shutdown_signal => {
                        break;
                    },
                    _ = interval.tick() => {
                        let statistics = context.database.read().get_data_volume_statistics();
                        context.raw_db_metrics.update(statistics);
                    },
                }
            }
        });
    }
}
