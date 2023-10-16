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

use std::sync::Arc;

use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use std::time::Duration;

use tokio::runtime::Runtime;
use tokio_util::sync::CancellationToken;

use crate::locks::{LockFactory, Mutex};

pub trait CancellableToken {
    /// Be aware that cancellation is not an atomic operation. It is possible
    /// for another thread running in parallel with a call to `cancel` to first
    /// receive `true` from `is_cancelled` on one child node, and then receive
    /// `false` from `is_cancelled` on another child node. However, once the
    /// call to `cancel` returns, all child nodes have been fully cancelled.
    fn cancel(&self);
}

/// Forward/delegate implementation of [`CancellableToken`] for Tokio's [`CancellationToken`]
impl CancellableToken for CancellationToken {
    fn cancel(&self) {
        self.cancel();
    }
}

pub struct NoopCancellationFuture;

impl Future for NoopCancellationFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<()> {
        Poll::Ready(())
    }
}

/// Noop implementation of the [`CancellableToken`]
pub struct NoopCancellableToken;

impl CancellableToken for NoopCancellableToken {
    fn cancel(&self) {}
}

/// A utility wrapper around a [`CancellableToken`] which will
/// cancel the task when dropped.
pub struct CancelOnDrop<CT: CancellableToken> {
    pub cancellation_token: CT,
}

impl<CT: CancellableToken> CancelOnDrop<CT> {
    pub fn new(cancellation_token: CT) -> Self {
        Self { cancellation_token }
    }
}

impl<CT: CancellableToken> Drop for CancelOnDrop<CT> {
    fn drop(&mut self) {
        self.cancellation_token.cancel();
    }
}

/// A scheduler for background tasks.
/// All schedules started with a specific scheduler should be stopped when its instance is dropped.
pub trait Scheduler {
    type CancellationTokenType: CancellableToken;
    /// Starts a periodic execution of the given task.
    /// The consecutive runs should be started at approximately the given intervals - however, if
    /// a particular run takes longer than the interval, then it should not be (in any way) aborted,
    /// and no concurrent run should be started - the schedule should simply be delayed.
    fn start_periodic(
        &self,
        duration: Duration,
        task: impl 'static + FnMut() + Send,
    ) -> Self::CancellationTokenType;
}

// TODO(metrics): Add a `MeasuredScheduler` decorator (mostly for: task run duration).

/// A no-op [`Scheduler`] (e.g. for test purposes).
pub struct NoopScheduler;

impl Scheduler for NoopScheduler {
    type CancellationTokenType = NoopCancellableToken;

    fn start_periodic(
        &self,
        _duration: Duration,
        _task: impl 'static + FnMut() + Send,
    ) -> Self::CancellationTokenType {
        NoopCancellableToken {}
    }
}

pub struct TokioScheduler {
    runtime: Arc<Runtime>,
}

impl TokioScheduler {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        Self { runtime }
    }
}

impl Scheduler for TokioScheduler {
    type CancellationTokenType = CancellationToken;

    fn start_periodic(
        &self,
        duration: Duration,
        mut task: impl 'static + FnMut() + Send,
    ) -> Self::CancellationTokenType {
        let cancellation_token = Self::CancellationTokenType::new();
        let cloned_token = cancellation_token.clone();
        self.runtime.spawn(async move {
            let mut interval = tokio::time::interval(duration);

            loop {
                tokio::select! {
                    _ = cloned_token.cancelled() => {
                        break;
                    },
                    _ = interval.tick() => {
                       task();
                    },
                }
            }
        });
        cancellation_token
    }
}

/// An auxiliary structure to be used along side a [`Scheduler`] in order to collect
/// [`CancellableToken`]s for the spawned tasks in order to autostop them on drop.
pub struct TaskTracker<CT: CancellableToken> {
    tasks: Vec<CancelOnDrop<CT>>,
}

impl<CT: CancellableToken> TaskTracker<CT> {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    pub fn track(&mut self, cancel_token: CT) {
        self.tasks.push(CancelOnDrop::new(cancel_token));
    }
}

impl<CT: CancellableToken> Default for TaskTracker<CT> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TokioSchedulerWithTaskTracker {
    scheduler: TokioScheduler,
    /// Note: the mutex is needed to satisfy [`Scheduler`]'s `start_periodic` signature.
    task_tracker: Mutex<TaskTracker<CancellationToken>>,
}

impl TokioSchedulerWithTaskTracker {
    pub fn new(runtime: Arc<Runtime>, lock_factory: LockFactory) -> Self {
        Self {
            scheduler: TokioScheduler::new(runtime),
            task_tracker: lock_factory
                .named("task_tracker")
                .new_mutex(TaskTracker::new()),
        }
    }
}

impl Scheduler for TokioSchedulerWithTaskTracker {
    type CancellationTokenType = CancellationToken;

    fn start_periodic(
        &self,
        duration: Duration,
        task: impl 'static + FnMut() + Send,
    ) -> Self::CancellationTokenType {
        let cancellation_token = self.scheduler.start_periodic(duration, task);
        self.task_tracker.lock().track(cancellation_token.clone());
        cancellation_token
    }
}
