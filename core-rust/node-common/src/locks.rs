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

use std::ops::{Deref, DerefMut};

use std::sync::Arc;

use crate::locks::MeasuredLockState::BeforeRelease;
use crate::metrics::{new_timer_vec, opts, AtDefaultRegistryExt};
use prometheus::{Histogram, HistogramVec, IntGauge, IntGaugeVec, Registry};
use std::thread;
use std::time::Instant;
use tracing::{error, info};

//==================================================================================================
// DEFINITION:
// A synchronization primitive is "panic-safe" if it does not allow any `panic!` to leave the
// subject of synchronization in an inconsistent state.
// Please note that every `panic!` that occurs while a lock is acquired for writing can interrupt a
// series of operations somewhere in the middle, where invariants are not kept. Hence, the
// implementations in this module will always trigger a preconfigured "service stopper" in such
// situations (in order to mitigate inconsistent data access from subsequent calls, possibly from
// other threads).
// The vanilla `parking_lot` synchronization primitives are not panic-safe in any way, i.e. they
// simply release the lock when the stack in unwinding (without shutting down the application or
// at least "poisoning" the lock).
//==================================================================================================

/// A lock factory facade.
/// Currently can be configured to provide the following features:
/// - Panic-safety (see the definition in a comment block above).
/// - Lock wait/hold timing measurements.
#[derive(Clone)]
pub struct LockFactory {
    name: String,
    stopper: Option<PanicSafetyApplicationStopper>,
    metrics: Option<LockFactoryMetrics>,
}

impl LockFactory {
    /// Creates a new lock factory.
    /// If left unconfigured, the locks returned by this instance will forward unchanged behaviors
    /// of their underlying [`parking_lot`] instances.
    /// The given `base_name` simply becomes the first segment of the constructed locks' names' (see
    /// [`Self::named()`]).
    ///
    /// Important factory API note:
    /// The actual lock-creating methods (i.e. the [`Self::new_*()`] family) *consume* the factory
    /// instance. This is by design, since we want to avoid 2 locks of the same name (since they
    /// could be confused in the log messages or the metrics). If you actually have a use-case for
    /// dynamically-created, unbounded, unidentified locks, you can still achieve it by cloning
    /// the factory instance - just make sure to unconfigure any features that could lead to
    /// confusion.
    pub fn new(base_name: impl Display) -> Self {
        Self {
            stopper: None,
            name: base_name.to_string(),
            metrics: None,
        }
    }

    /// Appends another segment to the dot-separated names of locks *created from this point on*.
    /// The names are used only for error-surfacing and metrics purposes.
    pub fn named(&self, segment: impl Display) -> Self {
        let mut derived = self.clone();
        derived.name = format!("{}.{}", derived.name, segment);
        derived
    }

    /// Configures the panic-safety behaviour of locks *created from this point on*.
    /// The implementations will reliably call the given [`stopper`] function exactly once
    /// on the first occurrence of "guard (of a state-modification lock) dropped while panicking".
    pub fn stopping_on_panic(&self, stopper: impl FnOnce() + Send + 'static) -> Self {
        let mut derived = self.clone();
        derived.stopper = Some(PanicSafetyApplicationStopper::wrap(stopper));
        derived
    }

    /// Unconfigures the [`Self::stopping_on_panic()`] of locks *created from this point on*.
    pub fn not_stopping_on_panic(&self) -> Self {
        let mut derived = self.clone();
        derived.stopper = None;
        derived
    }

    /// Configures the metrics collection for locks *created from this point on*.
    /// Each method returning a guard will measure:
    /// - a number of threads currently waiting for a guard to be returned.
    /// - time it took to wait for the guard.
    /// - a number of threads currently holding the guard (typically [0, 1], but can be higher e.g.
    ///   in case of a read lock).
    /// - time the guard was held.
    pub fn measured(&self, metrics_registry: &Registry) -> Self {
        let mut derived = self.clone();
        derived.metrics = Some(LockFactoryMetrics::new(metrics_registry));
        derived
    }

    /// Unconfigures the [`Self::measured()`] of locks *created from this point on*.
    pub fn not_measured(&self) -> Self {
        let mut derived = self.clone();
        derived.metrics = None;
        derived
    }

    /// Creates a new mutex with the current configuration.
    pub fn new_mutex<T>(self, value: T) -> Mutex<T> {
        Mutex {
            underlying: parking_lot::const_mutex(value),
            listener: self.into_listener(),
        }
    }

    /// Creates a new reader/writer lock with the current configuration.
    pub fn new_rwlock<T>(self, value: T) -> RwLock<T> {
        RwLock {
            underlying: parking_lot::const_rwlock(value),
            read_listener: self.named("read").not_stopping_on_panic().into_listener(),
            write_listener: self.named("write").into_listener(),
        }
    }

    /// Creates a new state lock with the current configuration.
    /// Note: this is a custom lock primitive, please consult its docs.
    pub fn new_state_lock<T>(self, value: T) -> StateLock<T> {
        StateLock {
            underlying: self.named("current").new_rwlock(()),
            value,
            access_non_locked_historical_listener: self
                .named("historical")
                .not_stopping_on_panic()
                .into_listener(),
        }
    }

    /// Turns the factory (i.e. its current configuration) into a [`LockListener`] which adds the
    /// requested features to the lock.
    fn into_listener(self) -> ActualLockListener {
        ChainLockListener {
            first: self
                .metrics
                .map(|metrics| metrics.create_listener_for(self.name.as_str())),
            second: self
                .stopper
                .map(|stopper| stopper.create_listener_for(self.name)),
        }
    }
}

/// A facade for a [`parking_lot::Mutex`] with a support for an arbitrary [`LockListener`].
pub struct Mutex<T> {
    underlying: parking_lot::Mutex<T>,
    listener: ActualLockListener,
}

impl<T> Mutex<T> {
    /// Delegates to the [`parking_lot::Mutex::lock()`].
    pub fn lock(&self) -> impl DerefMut<Target = T> + '_ {
        LockGuard::new(|| self.underlying.lock(), self.listener.clone())
    }
}

/// A facade for a [`parking_lot::RwLock`] with a support for an arbitrary [`LockListener`].
pub struct RwLock<T> {
    underlying: parking_lot::RwLock<T>,
    read_listener: ActualLockListener,
    write_listener: ActualLockListener,
}

impl<T> RwLock<T> {
    /// Delegates to the [`parking_lot::RwLockReadGuard::read()`].
    pub fn read(&self) -> impl Deref<Target = T> + '_ {
        LockGuard::new(|| self.underlying.read(), self.read_listener.clone())
    }

    /// Delegates to the [`parking_lot::RwLockReadGuard::write()`].
    pub fn write(&self) -> impl DerefMut<Target = T> + '_ {
        LockGuard::new(|| self.underlying.write(), self.write_listener.clone())
    }
}

/// A custom lock primitive guarding a "current state" of a value composed from an (immutable)
/// "historical" and (live) "current" parts of state.
/// The assumption is that the current state needs a classic [`RwLock`] access, while the historical
/// state can be accessed freely, without obtaining any lock.
/// The lock caller is responsible for distinguishing proper current vs historical access.
// TODO(future refactoring): It seems like the "weird lock" should not be needed if we had a DB
// interface which is more aware of its "current vs historical" nature. Maybe we will naturally go
// in that direction when introducing DB snapshotting.
pub struct StateLock<T> {
    underlying: RwLock<()>, // we use our own primitive to lock a marker for current state
    value: T,
    access_non_locked_historical_listener: ActualLockListener, // only for metrics
}

impl<T> StateLock<T> {
    /// Locks the current state for reading.
    /// This method should be used when caller needs a series of reads referring to the current
    /// state (it "freezes" the notion of "current").
    pub fn read_current(&self) -> impl Deref<Target = T> + '_ {
        StateLockGuard {
            underlying: self.underlying.read(),
            value: &self.value,
        }
    }

    /// Locks the current state for writing.
    /// This method should be used when caller wants to update the guarded value in a way which
    /// changes the notion of "current".
    /// Please note that this method deliberately returns [`Deref`] (not [`DerefMut`]), since it
    /// would create an undefined behaviour (`&` and `&mut` co-existing). The guarded value is
    /// assumed to use an interior mutability (i.e. expose mutating methods via `&`).
    pub fn write_current(&self) -> impl Deref<Target = T> + '_ {
        StateLockGuard {
            underlying: self.underlying.write(),
            value: &self.value,
        }
    }

    /// Returns a reference to the guarded value, without locking anything.
    /// This method should be used when the caller wants to interact selectively with pieces of the
    /// historical state, in a way known to be safe.
    /// Note: functionally, we could return a `&T` here directly, but returning a "guard" allows us
    /// to measure usage of this method (in the same way as we do for lock guards).
    pub fn access_non_locked_historical(&self) -> impl Deref<Target = T> + '_ {
        LockGuard::new(
            || &self.value,
            self.access_non_locked_historical_listener.clone(),
        )
    }
}

/// A read guard of a [`StateLock`].
pub struct StateLockGuard<'a, T, U> {
    #[allow(dead_code)] // only held to release the lock when dropped
    underlying: U,
    value: &'a T,
}

impl<'a, T: 'a, U> Deref for StateLockGuard<'a, T, U> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

// Only iternals below:

/// A static type of a [`LockListener`] which provides the extra features to locks produced by our
/// facade.
/// Current value can be interpreted as "a chain of 2 optional listeners: for metrics and for
/// panic-safety".
type ActualLockListener =
    ChainLockListener<Option<MetricsLockListener>, Option<PanicSafetyLockListener>>;

/// A generic lock-guard decorator with listener support.
/// Note: this struct is private; publicly, we operate on [`Deref`]/[`DerefMut`] traits (since the
/// caller only care about them - and in particular, this allows us to hide the entire
/// [`LockListener`] infra from the callers).
struct LockGuard<U, L: LockListener> {
    underlying: U,
    listener: L,
}

impl<U, L: LockListener> LockGuard<U, L> {
    /// Acquires the given lock while notifying the given listener on the consecutive stages.
    /// The lock is expressed as a generic function, so it can be universally used for many lock
    /// primitives.
    fn new(lock: impl FnOnce() -> U, mut listener: L) -> Self {
        listener.on_wait();
        let underlying = lock();
        listener.on_hold();
        Self {
            underlying,
            listener,
        }
    }
}

impl<T, U: Deref<Target = T>, L: LockListener> Deref for LockGuard<U, L> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.underlying.deref()
    }
}

impl<T, U: DerefMut<Target = T>, L: LockListener> DerefMut for LockGuard<U, L> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.underlying.deref_mut()
    }
}

impl<U, L: LockListener> Drop for LockGuard<U, L> {
    fn drop(&mut self) {
        self.listener.on_release();
    }
}

/// A wrapper around the complex "any thread-safe only-once function" type, used for the application
/// stopper dependency of our facade.
#[derive(Clone)]
struct PanicSafetyApplicationStopper(Arc<TakeOnce<Box<dyn FnOnce() + Send>>>);

impl PanicSafetyApplicationStopper {
    /// A wraps an arbitrary [`FnOnce`] into this thread-safe wrapper.
    pub fn wrap(function: impl FnOnce() + Send + 'static) -> Self {
        Self(Arc::new(TakeOnce::new(Box::new(function))))
    }

    /// Creates a [`LockListener`] which will call the wrapped function if a lock is released by
    /// being dropped while panicking.
    pub fn create_listener_for(&self, lock_name: String) -> PanicSafetyLockListener {
        PanicSafetyLockListener {
            stopper: self.clone(),
            lock_name: Arc::new(lock_name),
        }
    }
}

impl Deref for PanicSafetyApplicationStopper {
    type Target = TakeOnce<Box<dyn FnOnce() + Send>>;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

/// A synchronized box allowing to take the ownership of its contents only once, on first access.
struct TakeOnce<T> {
    // Note: ironically, the implementation uses a raw `parking_lot::Mutex`, which is not wrapped in
    // our facade (because of chicken-and-egg reasons). However, this is fine, since we never block
    // on the mutex (i.e. we only `try_lock()`, which reduces to a single CAS).
    mutex: parking_lot::Mutex<Option<T>>,
}

impl<T> TakeOnce<T> {
    /// Created a synchronized take-once-box with the given contents.
    pub fn new(value: T) -> Self {
        Self {
            mutex: parking_lot::const_mutex(Some(value)),
        }
    }

    /// Returns the contents if this is the very first invocation on this instance, or [`None`]
    /// otherwise.
    pub fn take(&self) -> Option<T> {
        if let Some(mut option) = self.mutex.try_lock() {
            option.take()
        } else {
            None
        }
    }
}

/// A generic stateful listener of a single *lock interaction* lifecycle of a specific lock.
///
/// Implementation shortcut note:
/// We use a couple of listeners, and technically, each lock should own a listener *factory*, which
/// it would use to create a new stateful listener dedicated to a specific "lock-hold-release" cycle
/// (an interaction between a lock and a thread). However, this would require a factory
/// implementation for each listener, which only bloats the code. Instead, for brevity, our locks
/// own actual [`LockListener`]s in an "initialized, unused" state, which they clone for each
/// interaction (a.k.a. "prototype").
trait LockListener {
    /// Notices that a thread started waiting to acquire the lock.
    fn on_wait(&mut self) {}

    /// Notices that a thread has acquired the lock and now holds it.
    fn on_hold(&mut self) {}

    /// Notices that a thread has released the lock.
    fn on_release(&mut self) {}
}

/// A [`LockListener`] to be used by all "lock write guards" which need to trigger an "application
/// stopper" on drop, if the current thread is panicking.
#[derive(Clone)]
struct PanicSafetyLockListener {
    stopper: PanicSafetyApplicationStopper,
    lock_name: Arc<String>, // for logging purposes only; Arc<> is used for cheap clone of the immutable String
}

impl LockListener for PanicSafetyLockListener {
    fn on_release(&mut self) {
        if !thread::panicking() {
            return;
        }
        if let Some(stopper) = self.stopper.take() {
            error!(
                "a write guard of {} was dropped while panicking; stopping",
                self.lock_name
            );
            stopper();
        } else {
            info!(
                "a write guard of {} was dropped during stopping; ignoring",
                self.lock_name
            );
        }
    }
}

/// A trivial chain of 2 [`LockListener`]s.
#[derive(Clone)]
struct ChainLockListener<D1, D2> {
    first: D1,
    second: D2,
}

impl<D1: LockListener, D2: LockListener> LockListener for ChainLockListener<D1, D2> {
    fn on_wait(&mut self) {
        self.first.on_wait();
        self.second.on_wait();
    }

    fn on_hold(&mut self) {
        self.first.on_hold();
        self.second.on_hold();
    }

    fn on_release(&mut self) {
        self.first.on_release();
        self.second.on_release();
    }
}

/// A [`LockListener`] bumping the [`LockFactoryMetrics`] scoped at a specific lock (via the
/// [`LockFactoryMetrics::LOCK_NAME_LABEL'] label).
#[derive(Clone)]
struct MetricsLockListener {
    waiting_threads: IntGauge,
    wait: Histogram,
    holding_threads: IntGauge,
    hold: Histogram,
    state: MeasuredLockState,
}

/// An internal state of the [`MetricsLockListener`]'s lifecycle.
#[derive(Debug)]
enum MeasuredLockState {
    BeforeWait,
    BeforeHold { started_waiting_at: Instant },
    BeforeRelease { started_holding_at: Instant },
    Dead,
}

impl Clone for MeasuredLockState {
    fn clone(&self) -> Self {
        assert!(
            matches!(self, MeasuredLockState::BeforeWait),
            "not an initial state: {:?}",
            self
        );
        MeasuredLockState::BeforeWait
    }
}

impl LockListener for MetricsLockListener {
    fn on_wait(&mut self) {
        assert!(
            matches!(self.state, MeasuredLockState::BeforeWait),
            "unexpected state: {:?}",
            self.state
        );
        self.waiting_threads.inc();
        self.state = MeasuredLockState::BeforeHold {
            started_waiting_at: Instant::now(),
        };
    }

    fn on_hold(&mut self) {
        let MeasuredLockState::BeforeHold { started_waiting_at } = self.state else {
            panic!("unexpected state: {:?}", self.state)
        };
        self.waiting_threads.dec();
        self.holding_threads.inc();
        let started_holding_at = Instant::now();
        let wait_duration = started_holding_at.duration_since(started_waiting_at);
        self.wait.observe(wait_duration.as_secs_f64());
        self.state = MeasuredLockState::BeforeRelease { started_holding_at };
    }

    fn on_release(&mut self) {
        let BeforeRelease { started_holding_at } = self.state else {
            panic!("unexpected state: {:?}", self.state)
        };
        self.holding_threads.dec();
        self.hold
            .observe(started_holding_at.elapsed().as_secs_f64());
        self.state = MeasuredLockState::Dead;
    }
}

impl<L: LockListener> LockListener for Option<L> {
    fn on_wait(&mut self) {
        if let Some(underlying) = self {
            underlying.on_wait();
        }
    }

    fn on_hold(&mut self) {
        if let Some(underlying) = self {
            underlying.on_hold();
        }
    }

    fn on_release(&mut self) {
        if let Some(underlying) = self {
            underlying.on_release();
        }
    }
}

/// A set of metrics applicable to all "locks" (understood as: "things for which you wait to give
/// you a lock guard, which you later release by dropping").
#[derive(Debug, Clone)]
struct LockFactoryMetrics {
    waiting_threads: IntGaugeVec,
    wait: HistogramVec,
    holding_threads: IntGaugeVec,
    hold: HistogramVec,
}

impl LockFactoryMetrics {
    const LOCK_NAME_LABEL: &'static str = "lock";

    /// Registers the metrics in the given registry.
    pub fn new(registry: &Registry) -> Self {
        Self {
            waiting_threads: IntGaugeVec::new(
                opts(
                    "locks_waiting_threads",
                    "A number of threads currently waiting to acquire a specific lock.",
                ),
                &[Self::LOCK_NAME_LABEL],
            ).registered_at(registry),
            wait: new_timer_vec(
                opts(
                    "locks_acquire",
                    "Time spent waiting to acquire a specific lock.",
                ),
                &[Self::LOCK_NAME_LABEL],
                vec![
                    0.0001, 0.0005, 0.002, 0.01, 0.05, 0.2, 1.0, 5.0,
                ],
            ).registered_at(registry),
            holding_threads: IntGaugeVec::new(
                opts(
                    "locks_holding_threads",
                    "A number of threads currently holding a specific lock (may actually be >1 e.g. for a reader lock).",
                ),
                &[Self::LOCK_NAME_LABEL],
            ).registered_at(registry),
            hold: new_timer_vec(
                opts(
                    "locks_hold",
                    "Time spent holding a specific lock.",
                ),
                &[Self::LOCK_NAME_LABEL],
                vec![
                    0.0001, 0.0005, 0.002, 0.01, 0.05, 0.2, 1.0, 5.0,
                ],
            ).registered_at(registry)
        }
    }

    /// Creates a [`LockListener`] which will update the metrics scoped at a particular lock (using
    /// the [`Self::LOCK_NAME_LABEL`] label with the given value).
    /// Please note that a simple [`Mutex`] uses just a single metrics listener, while e.g. an
    /// [`RwLock`] needs to separately track read vs write locks - this can be achieved by creating
    /// 2 listeners with differently suffixed `lock_name`s.
    pub fn create_listener_for(&self, lock_name: &str) -> MetricsLockListener {
        let labels = &[lock_name];
        MetricsLockListener {
            waiting_threads: self.waiting_threads.with_label_values(labels),
            wait: self.wait.with_label_values(labels),
            holding_threads: self.holding_threads.with_label_values(labels),
            hold: self.hold.with_label_values(labels),
            state: MeasuredLockState::BeforeWait,
        }
    }
}
