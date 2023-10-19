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

use std::cell::RefCell;
use std::fmt::Display;

use std::rc::Rc;
use std::sync::Arc;

use prometheus::{HistogramVec, Registry};
use std::time::{Duration, Instant};

use tokio::runtime::Runtime;
use tokio_util::sync::CancellationToken;

use crate::metrics::{new_timer_vec, opts, AtDefaultRegistryExt, MetricLabel, TakesMetricLabels};

/// A transient starter of periodically-executed tasks.
///
/// Important scheduler API note:
/// Despite the friendly and short name ("scheduler"), the struct defined below is more of an
/// "immutable (hierarchical) builder of task schedules". Each configuration method returns a new
/// "child" scheduler instance which applies the configuration from that point downward in the
/// hierarchy.
/// The actual task-starting methods (e.g. [`Self::start_periodic()`]) *consume* the scheduler
/// instance. This is by design, since we want to avoid 2 scheduled tasks of the same name (i.e.
/// they could be confused in the log messages or the metrics). If you actually have a use-case for
/// dynamically-created, unbounded, unidentified tasks, you can still achieve it by cloning
/// the factory instance - just make sure to un-configure any features that rely on unique names.
pub struct Scheduler<S, T, M> {
    name: String,
    spawner: Rc<S>,
    running_task_tracker: Rc<T>,
    metrics: Arc<M>, // the thread-safety is only needed for metrics, since only they are referenced from spawned tasks
}

impl Scheduler<NoopSpawner, NoTracking, NoMetrics> {
    /// Starts the build of a new, "bare" schedule.
    /// Note: such instance will be no-op until you configure an actual [`Spawner`] (see e.g.
    /// [`Self::use_tokio()`]), suitable probably only for tests.
    /// The given `base_name` simply becomes the first segment of the constructed tasks' names' (see
    /// [`Self::named()`]).
    pub fn new(base_name: impl Display) -> Self {
        Self {
            name: base_name.to_string(),
            spawner: Rc::new(NoopSpawner),
            running_task_tracker: Rc::new(NoTracking),
            metrics: Arc::new(NoMetrics),
        }
    }
}

impl<S, T, M> Scheduler<S, T, M> {
    /// Configures the given tokio [`Runtime`] to be used for spawning tasks *created from this
    /// point on*.
    pub fn use_tokio<'r>(&self, runtime: &'r Runtime) -> Scheduler<TokioSpawner<'r>, T, M> {
        Scheduler {
            spawner: Rc::new(TokioSpawner::new(runtime)),
            name: self.name.clone(),
            running_task_tracker: self.running_task_tracker.clone(),
            metrics: self.metrics.clone(),
        }
    }

    /// Appends another segment to the dot-separated names of tasks *created from this point on*.
    /// The names are used only for error-surfacing and metrics purposes.
    pub fn named(&self, segment: impl Display) -> Self {
        Scheduler {
            spawner: self.spawner.clone(),
            name: format!("{}.{}", self.name, segment),
            running_task_tracker: self.running_task_tracker.clone(),
            metrics: self.metrics.clone(),
        }
    }

    /// Configures the auto-tracking of the tasks *created from this point on*.
    /// Every [`RunningTask`] returned by the task-starting method will also be reported to the
    /// internal [`UntilDropTracker`] (which can be obtained upon finalization of the schedules'
    /// build, in order to keep all schedules alive - see [`Self::into_task_tracker()`]).
    pub fn track_running_tasks(&self) -> Scheduler<S, UntilDropTracker, M> {
        Scheduler {
            spawner: self.spawner.clone(),
            name: self.name.clone(),
            running_task_tracker: Rc::new(UntilDropTracker::default()),
            metrics: self.metrics.clone(),
        }
    }

    /// Configures the metrics collection for tasks *created from this point on*.
    pub fn measured(&self, metrics_registry: &Registry) -> Scheduler<S, T, SchedulerMetrics> {
        Scheduler {
            spawner: self.spawner.clone(),
            name: self.name.clone(),
            running_task_tracker: self.running_task_tracker.clone(),
            metrics: Arc::new(SchedulerMetrics::new(metrics_registry)),
        }
    }
}

impl<S: Spawner, T: Tracker, M: Metrics> Scheduler<S, T, M> {
    /// Starts a periodic execution of the given task.
    /// The consecutive runs should be started at approximately the given intervals - however, if
    /// a particular run takes longer than the interval, then it should not be (in any way) aborted,
    /// and no concurrent run should be started - the schedule should simply be delayed.
    /// The task will keep running periodically until the returned [`RunningTask`] is dropped (to
    /// be specific - until the *last* of its clones is dropped).
    /// Note: it is important to either keep the returned instance alive, or configure the
    /// auto-tracking feature (see [`Self::track_running_tasks()`]).
    pub fn start_periodic(
        self,
        interval: Duration,
        mut task: impl 'static + FnMut() + Send,
    ) -> RunningTask {
        let measured = move || self.metrics.execute_measured(&mut task, &self.name);
        let cancellation_token = CancellationToken::new();
        let running_task = RunningTask::new(cancellation_token.clone());
        self.spawner.spawn(interval, measured, cancellation_token);
        self.running_task_tracker.track(running_task.clone());
        running_task
    }
}

impl<S: Spawner, M: Metrics> Scheduler<S, UntilDropTracker, M> {
    /// Finalizes the build of all schedules using this instance (or its children derived by
    /// configuration changes) and returns a tracker holding all [`RunningTask`] handlers.
    /// Please note that this is only available when the actual task-tracking was enabled (see
    /// [`Self::track_running_tasks()`]).
    pub fn into_task_tracker(self) -> UntilDropTracker {
        Rc::into_inner(self.running_task_tracker)
            .expect("some child Scheduler instance still lives; cannot finalize")
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

/// A [`RunningTask`] tracker maintained internally by a [`Scheduler`] during its "build" phase,
/// and returned on finalization.
pub trait Tracker {
    /// Takes ownership of the given task handle.
    fn track(&self, running_task: RunningTask);
}

/// A no-op [`Tracker`], used when no auto-tracking is configured.
/// Clients must manually hold all task handles returned while this tracker is effective - otherwise
/// the tasks will be cancelled immediately.
pub struct NoTracking;

impl Tracker for NoTracking {
    fn track(&self, _running_task: RunningTask) {
        // intentionally empty
    }
}

/// A [`Tracker`] implementation enabled by [`Scheduler#track_running_tasks()`] method in order
/// to collect [`RunningTask`]s and keep them alive until this single tracker instance is dropped.
pub struct UntilDropTracker {
    running_tasks: RefCell<Vec<RunningTask>>,
}

impl Default for UntilDropTracker {
    fn default() -> Self {
        Self {
            running_tasks: RefCell::new(Vec::new()),
        }
    }
}

impl Tracker for UntilDropTracker {
    fn track(&self, running_task: RunningTask) {
        self.running_tasks.borrow_mut().push(running_task);
    }
}

/// A low-level spawner of threads (or other async processing primitives) used by the scheduler.
pub trait Spawner {
    /// Starts an asynchronous periodic execution of the given task at the given intervals, until
    /// [`CancellationToken::cancelled()`].
    fn spawn(
        &self,
        interval: Duration,
        task: impl 'static + FnMut() + Send,
        cancellation_token: CancellationToken,
    );
}

/// A no-op [`Spawner`], effective until a proper async execution backend is configured for a
/// scheduler.
/// No tasks are actually ever executed with this spawner.
pub struct NoopSpawner;

impl Spawner for NoopSpawner {
    fn spawn(
        &self,
        _interval: Duration,
        _task: impl 'static + FnMut() + Send,
        _cancellation_token: CancellationToken,
    ) {
        // intentionally empty
    }
}

/// A [`Spawner`] backed by a tokio runtime.
#[derive(Debug, Clone)]
pub struct TokioSpawner<'r> {
    runtime: &'r Runtime,
}

impl<'r> TokioSpawner<'r> {
    fn new(runtime: &'r Runtime) -> Self {
        Self { runtime }
    }
}

impl<'r> Spawner for TokioSpawner<'r> {
    fn spawn(
        &self,
        interval: Duration,
        mut task: impl 'static + FnMut() + Send,
        cancellation_token: CancellationToken,
    ) {
        self.runtime.spawn(async move {
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

/// An internal delegate of a scheduler, handling measurements around each task execution.
/// Note: the `Sync + Send + 'static` is required here, since we need to refer to metrics from
/// another thread (the one running the task).
pub trait Metrics: Sync + Send + 'static {
    /// Executes the given function synchronously while recording arbitrary measurements related
    /// to this particular execution.
    fn execute_measured(&self, task: &mut impl FnMut(), name: impl MetricLabel);
}

/// A no-op [`Metrics`] which does not measure anything.
pub struct NoMetrics;

impl Metrics for NoMetrics {
    fn execute_measured(&self, task: &mut impl FnMut(), _name: impl MetricLabel) {
        task()
    }
}

/// A [`Metrics`] implementation following our Prometheus conventions.
// TODO(metrics): Add more: counter of execution overlaps, currently stalled executions, etc.
#[derive(Debug, Clone)]
pub struct SchedulerMetrics {
    task_execute: HistogramVec,
}

impl SchedulerMetrics {
    const TASK_NAME_LABEL: &'static str = "task";

    fn new(registry: &Registry) -> Self {
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
}

impl Metrics for SchedulerMetrics {
    fn execute_measured(&self, task: &mut impl FnMut(), name: impl MetricLabel) {
        let started_execution_at = Instant::now();
        task();
        self.task_execute
            .with_label(name)
            .observe(started_execution_at.elapsed().as_secs_f64());
    }
}
