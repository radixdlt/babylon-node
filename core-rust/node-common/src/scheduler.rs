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

use std::time::Duration;
use tokio::{runtime::Runtime, sync::oneshot};

pub struct ShutdownOnDrop {
    pub shutdown_signal_sender: Option<oneshot::Sender<()>>,
}

impl ShutdownOnDrop {
    pub fn new(shutdown_signal_sender: oneshot::Sender<()>) -> Self {
        Self {
            shutdown_signal_sender: Some(shutdown_signal_sender),
        }
    }
}

impl Drop for ShutdownOnDrop {
    fn drop(&mut self) {
        if let Some(sender) = self.shutdown_signal_sender.take() {
            // Using `let _ =` to ignore send errors.
            let _ = sender.send(());
        }
    }
}

/// A scheduler for background tasks.
/// All schedules started with a specific scheduler should be stopped when its instance is dropped.
pub trait Scheduler {
    /// Starts a periodic execution of the given task.
    /// The consecutive runs should be started at approximately the given intervals - however, if
    /// a particular run takes longer than the interval, then it should not be (in any way) aborted,
    /// and no concurrent run should be started - the schedule should simply be delayed.
    fn start_periodic_advanced(
        &self,
        duration: Duration,
        task: impl 'static + FnMut() + Send,
        shutdown_signal: oneshot::Receiver<()>,
    );

    /// Starts a periodic execution of the given task.
    /// Unlike the `_advanced` method, it will take ownership of the shutdown signal and call it when the [`Scheduler`] is dropped.
    fn start_periodic(&mut self, duration: Duration, task: impl 'static + FnMut() + Send);
}

// TODO(metrics): Add a `MeasuredScheduler` decorator (mostly for: task run duration).

/// A no-op [`Scheduler`] (e.g. for test purposes).
#[derive(Default)]
pub struct NoopScheduler;

impl Scheduler for NoopScheduler {
    fn start_periodic_advanced(
        &self,
        _duration: Duration,
        mut _task: impl 'static + FnMut() + Send,
        _shutdown_signal: oneshot::Receiver<()>,
    ) {
    }

    fn start_periodic(&mut self, _duration: Duration, _task: impl 'static + FnMut() + Send) {}
}

pub struct TokioScheduler {
    runtime: Arc<Runtime>,
    tasks: Vec<ShutdownOnDrop>,
}

impl TokioScheduler {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        Self {
            runtime,
            tasks: Vec::new(),
        }
    }
}

impl Scheduler for TokioScheduler {
    fn start_periodic_advanced(
        &self,
        duration: Duration,
        mut task: impl 'static + FnMut() + Send,
        shutdown_signal: oneshot::Receiver<()>,
    ) {
        self.runtime.spawn(async move {
            let mut shutdown_signal = shutdown_signal;
            let mut interval = tokio::time::interval(duration);

            loop {
                tokio::select! {
                    _ = &mut shutdown_signal => {
                        break;
                    },
                    _ = interval.tick() => {
                       task();
                    },
                }
            }
        });
    }

    fn start_periodic(&mut self, duration: Duration, mut task: impl 'static + FnMut() + Send) {
        let (shutdown_signal_sender, shutdown_signal_receiver) = oneshot::channel::<()>();
        self.start_periodic_advanced(
            duration,
            move || {
                task();
            },
            shutdown_signal_receiver,
        );
        self.tasks.push(ShutdownOnDrop::new(shutdown_signal_sender));
    }
}
