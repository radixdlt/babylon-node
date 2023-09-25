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

use std::ops::{Deref, DerefMut};

use std::sync::Arc;

use std::thread;
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

/// A factory of panic-safe lock primitives.
pub struct LockFactory {
    stopper: Arc<TakeOnce<Box<dyn FnOnce() + Send>>>,
    name: String,
}

impl LockFactory {
    /// Creates a new factory, which will reliably call the given [`stopper`] function exactly once
    /// on the first occurrence of "lock guard dropped while panicking" (on any of the lock
    /// primitives created by this factory).
    pub fn new(stopper: impl FnOnce() + Send + 'static) -> Self {
        Self {
            stopper: Arc::new(TakeOnce::new(Box::new(stopper))),
            name: "".to_string(),
        }
    }

    /// Appends another segment to the dot-separated names of the lock primitives created from this
    /// point on.
    /// The names are used only for error-surfacing purposes.
    pub fn named(&self, segment: impl Into<String>) -> Self {
        let segment = segment.into();
        Self {
            stopper: self.stopper.clone(),
            name: if self.name.is_empty() {
                segment
            } else {
                format!("{}.{}", self.name, segment)
            },
        }
    }

    /// Creates a new mutex, giving it the currently configured name.
    pub fn new_mutex<T>(&self, value: T) -> Mutex<T> {
        Mutex {
            underlying: parking_lot::const_mutex(value),
            panic_drop_handler: PanicDropHandler::new(self),
        }
    }

    /// Creates a new RW-lock, giving it the currently configured name.
    pub fn new_rwlock<T>(&self, value: T) -> RwLock<T> {
        RwLock {
            underlying: parking_lot::const_rwlock(value),
            panic_drop_handler: PanicDropHandler::new(self),
        }
    }
}

/// A panic-safe facade for a [`parking_lot::Mutex`].
pub struct Mutex<T> {
    underlying: parking_lot::Mutex<T>,
    panic_drop_handler: PanicDropHandler,
}

impl<T> Mutex<T> {
    /// Delegates to the [`parking_lot::Mutex::lock()`], but returns a panic-safe guard.
    pub fn lock(&self) -> MutexGuard<'_, T> {
        MutexGuard {
            underlying: self.underlying.lock(),
            panic_drop_handler: &self.panic_drop_handler,
        }
    }
}

/// A panic-safe facade for a [`parking_lot::MutexGuard`].
pub struct MutexGuard<'a, T> {
    underlying: parking_lot::MutexGuard<'a, T>,
    panic_drop_handler: &'a PanicDropHandler,
}

impl<'a, T: 'a> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.underlying.deref()
    }
}

impl<'a, T: 'a> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.underlying.deref_mut()
    }
}

impl<'a, T> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        self.panic_drop_handler.on_drop();
    }
}

/// A panic-safe facade for a [`parking_lot::RwLock`].
pub struct RwLock<T> {
    underlying: parking_lot::RwLock<T>,
    panic_drop_handler: PanicDropHandler,
}

impl<T> RwLock<T> {
    pub fn for_testing(value: T) -> Self {
        Self {
            underlying: parking_lot::const_rwlock(value),
            panic_drop_handler: PanicDropHandler::for_testing(),
        }
    }
}

impl<T> RwLock<T> {
    /// Delegates to the [`parking_lot::RwLockReadGuard::read()`], but returns a panic-safe guard.
    pub fn read(&self) -> RwLockReadGuard<'_, T> {
        RwLockReadGuard {
            underlying: self.underlying.read(),
        }
    }

    /// Delegates to the [`parking_lot::RwLockReadGuard::write()`], but returns a panic-safe guard.
    pub fn write(&self) -> RwLockWriteGuard<'_, T> {
        RwLockWriteGuard {
            underlying: self.underlying.write(),
            panic_drop_handler: &self.panic_drop_handler,
        }
    }
}

/// A panic-safe facade for a [`parking_lot::RwLockReadGuard`].
pub struct RwLockReadGuard<'a, T> {
    underlying: parking_lot::RwLockReadGuard<'a, T>,
}

impl<'a, T: 'a> Deref for RwLockReadGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.underlying.deref()
    }
}

impl<'a, T> Drop for RwLockReadGuard<'a, T> {
    fn drop(&mut self) {
        // The impl here is deliberately no-op:
        // The lock guard will be dropped (i.e. lock released) on its own, and in case of reading,
        // we do not need to abort the process (since unmodified state means consistent state).
    }
}

/// A panic-safe facade for a [`parking_lot::RwLockWriteGuard`].
pub struct RwLockWriteGuard<'a, T> {
    underlying: parking_lot::RwLockWriteGuard<'a, T>,
    panic_drop_handler: &'a PanicDropHandler,
}

impl<'a, T: 'a> Deref for RwLockWriteGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.underlying.deref()
    }
}

impl<'a, T: 'a> DerefMut for RwLockWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.underlying.deref_mut()
    }
}

impl<'a, T> Drop for RwLockWriteGuard<'a, T> {
    fn drop(&mut self) {
        self.panic_drop_handler.on_drop();
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
    pub fn new(value: T) -> Self {
        Self {
            mutex: parking_lot::const_mutex(Some(value)),
        }
    }

    pub fn taken() -> Self {
        Self {
            mutex: parking_lot::const_mutex(None),
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

/// An implementation delegate for all "lock write guards" which need to trigger a "service stopper"
/// on drop, if the current thread is panicking.
struct PanicDropHandler {
    stopper: Arc<TakeOnce<Box<dyn FnOnce() + Send>>>,
    name: String,
}

impl PanicDropHandler {
    fn new(factory: &LockFactory) -> Self {
        Self {
            stopper: factory.stopper.clone(),
            name: factory.name.clone(),
        }
    }

    pub fn for_testing() -> Self {
        Self {
            stopper: Arc::new(TakeOnce::taken()),
            name: "test".to_owned(),
        }
    }

    /// Attempts to invoke the contents of [`stopper`] if the current thread is panicking. Logs the
    /// attempt's outcome.
    pub fn on_drop(&self) {
        if !thread::panicking() {
            return;
        }
        if let Some(stopper) = self.stopper.take() {
            error!(
                "a write guard of {} was dropped while panicking; stopping",
                self.name
            );
            stopper();
        } else {
            info!(
                "a write guard of {} was dropped during stopping; ignoring",
                self.name
            );
        }
    }
}
