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

use std::fmt::Display;
use std::sync::Arc;

use prometheus::{HistogramVec, Registry};
use std::time::{Duration, Instant};

use tokio::runtime::Runtime;
use tokio_util::sync::CancellationToken;

use crate::locks::{LockFactory, Mutex};
use crate::metrics::{new_timer_vec, opts, AtDefaultRegistryExt};

/// A transient starter of periodically-executed tasks.
///
/// Important scheduler API note:
/// The actual task-starting methods (e.g. [`Self::start_periodic()`]) *consume* the scheduler
/// instance. This is by design, since we want to avoid 2 scheduled tasks of the same name (i.e.
/// they could be confused in the log messages or the metrics). If you actually have a use-case for
/// dynamically-created, unbounded, unidentified tasks, you can still achieve it by cloning
/// the factory instance - just make sure to unconfigure any features that rely on unique names.
#[derive(Clone)]
pub struct Scheduler<'a> {
    runtime: Option<&'a Runtime>,
    name: String,
    running_task_tracker: Option<&'a RunningTaskTracker>,
    metrics: Option<SchedulerMetrics>,
}

impl<'a> Scheduler<'a> {
    /// Creates a new scheduler instance using the given tokio `runtime`.
    /// The given `base_name` simply becomes the first segment of the constructed tasks' names' (see
    /// [`Self::named()`]).
    pub fn new(runtime: &'a Runtime, base_name: impl Display) -> Self {
        Self {
            runtime: Some(runtime),
            name: base_name.to_string(),
            running_task_tracker: None,
            metrics: None,
        }
    }

    /// Creates a mock instance for testing purposes.
    /// The "scheduled" tasks will not actually be started.
    pub fn for_testing() -> Self {
        Self {
            runtime: None,
            name: "testing".to_string(),
            running_task_tracker: None,
            metrics: None,
        }
    }

    /// Appends another segment to the dot-separated names of tasks *created from this point on*.
    /// The names are used only for error-surfacing and metrics purposes.
    pub fn named(&self, segment: impl Display) -> Self {
        let mut derived = self.clone();
        derived.name = format!("{}.{}", derived.name, segment);
        derived
    }

    /// Configures the auto-tracking of the tasks *created from this point on*.
    /// Every [`RunningTask`] returned by the task-starting method will also be reported to the
    /// given tracker (which may be easier for the application's boot-up logic to keep alive).
    pub fn track_running_tasks(&self, running_task_tracker: &'a RunningTaskTracker) -> Self {
        let mut derived = self.clone();
        derived.running_task_tracker = Some(running_task_tracker);
        derived
    }

    /// Configures the metrics collection for tasks *created from this point on*.
    pub fn measured(&self, metrics_registry: &Registry) -> Self {
        let mut derived = self.clone();
        derived.metrics = Some(SchedulerMetrics::new(metrics_registry));
        derived
    }

    /// Starts a periodic execution of the given task.
    /// The consecutive runs should be started at approximately the given intervals - however, if
    /// a particular run takes longer than the interval, then it should not be (in any way) aborted,
    /// and no concurrent run should be started - the schedule should simply be delayed.
    /// The task will keep running periodically until the returned [`RunningTask`] is dropped (to
    /// be specific - until the *last* of its clones is dropped).
    /// Note: it is important to either keep the returned value alive, or configure the
    /// auto-tracking feature (see [`Self::track_running_tasks()`]).
    pub fn start_periodic(
        self,
        interval: Duration,
        task: impl 'static + FnMut() + Send,
    ) -> RunningTask {
        let cancellation_token = CancellationToken::new();
        let running_task = RunningTask::new(cancellation_token.clone());
        let Some(runtime) = self.runtime else {
            // No runtime configured - we only fake the running, for test purposes:
            return running_task;
        };
        // The `if` below seems larger than required - this is for "incompatible types" reasons
        if let Some(metrics) = self.metrics {
            let task = metrics.wrap(task, self.name.as_str());
            Self::spawn_periodic_future(runtime, interval, task, cancellation_token)
        } else {
            Self::spawn_periodic_future(runtime, interval, task, cancellation_token)
        };
        if let Some(task_tracker) = self.running_task_tracker {
            task_tracker.track(running_task.clone());
        }
        running_task
    }

    fn spawn_periodic_future(
        runtime: &Runtime,
        interval: Duration,
        mut task: impl 'static + FnMut() + Send,
        cancellation_token: CancellationToken,
    ) {
        runtime.spawn(async move {
            let mut interval = tokio::time::interval(interval);
            loop {
                tokio::select! {
                    _ = cancellation_token.cancelled() => {
                        break;
                    },
                    _ = interval.tick() => {
                        task();
                    },
                }
            }
        });
    }
}

/// A handle representing a running task started using the scheduler.
/// Dropping *all* its clones cancels any subsequent executions of the task.
#[derive(Clone)]
pub struct RunningTask {
    cancellation_token: Arc<CancellationToken>,
}

impl RunningTask {
    fn new(cancellation_token: CancellationToken) -> Self {
        Self {
            cancellation_token: Arc::new(cancellation_token),
        }
    }
}

impl Drop for RunningTask {
    fn drop(&mut self) {
        if let Some(cancellation_token) = Arc::get_mut(&mut self.cancellation_token) {
            cancellation_token.cancel();
        }
    }
}

/// An auxiliary structure to be used with the [`Scheduler#track_running_tasks()`] method in order
/// to collect [`RunningTask`]s and keep them alive until this single tracker instance is dropped.
pub struct RunningTaskTracker {
    running_tasks: Mutex<Vec<RunningTask>>,
}

impl RunningTaskTracker {
    /// Creates a tracker.
    /// The [`LockFactory`] is used only to allow for interior mutability.
    pub fn new(factory: LockFactory) -> Self {
        Self {
            running_tasks: factory.new_mutex(Vec::new()),
        }
    }

    fn track(&self, running_task: RunningTask) {
        self.running_tasks.lock().push(running_task);
    }
}

/// A set of metrics for scheduled tasks / executions.
// TODO(metrics): Add more: counter of execution overlaps, currently stalled executions, etc.
#[derive(Debug, Clone)]
struct SchedulerMetrics {
    task_execute: HistogramVec,
}

impl SchedulerMetrics {
    const TASK_NAME_LABEL: &'static str = "task";

    /// Registers the metrics in the given registry.
    pub fn new(registry: &Registry) -> Self {
        Self {
            task_execute: new_timer_vec(
                opts(
                    "scheduler_task_execute",
                    "Time spent executing a single run of a specific task.",
                ),
                &[Self::TASK_NAME_LABEL],
                vec![0.001, 0.005, 0.02, 0.1, 0.5, 2.0, 5.0],
            )
            .registered_at(registry),
        }
    }

    /// Wraps the given task in a measuring decorator.
    pub fn wrap(
        &self,
        mut task: impl 'static + FnMut() + Send,
        name: &str,
    ) -> impl 'static + FnMut() + Send {
        let labels = &[name];
        let task_execute = self.task_execute.with_label_values(labels);
        move || {
            let started_execution_at = Instant::now();
            task();
            task_execute.observe(started_execution_at.elapsed().as_secs_f64());
        }
    }
}
