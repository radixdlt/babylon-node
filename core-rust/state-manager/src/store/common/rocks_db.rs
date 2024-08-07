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

use crate::engine_prelude::*;
use rocksdb::checkpoint::Checkpoint;
use rocksdb::{
    AsColumnFamilyRef, ColumnFamily, DBPinnableSlice,
    IteratorMode, Snapshot, WriteBatch, DB,
};

use std::path::PathBuf;

/// A redefined RocksDB's "key and value bytes" tuple (the original one lives in a private module).
pub type KVBytes = (Box<[u8]>, Box<[u8]>);

/// A trait capturing the common read methods present both in a "direct" RocksDB instance and in its
/// snapshots.
///
/// The library we use (a thin C wrapper, really) does not introduce this trivial and natural trait
/// itself, while we desperately need it to abstract the DB-reading code from the actual source of
/// data.
///
/// A note on changed error handling:
/// The original methods typically return [`Result`]s. Our trait assumes panics instead, since we
/// treat all database access errors as fatal anyways.
pub trait ReadableRocks {
    /// Resolves the column family by name.
    fn cf_handle(&self, name: &str) -> &ColumnFamily;

    /// Starts iteration over key-value pairs, according to the given [`IteratorMode`].
    fn iterator_cf(
        &self,
        cf: &impl AsColumnFamilyRef,
        mode: IteratorMode,
    ) -> Box<dyn Iterator<Item=KVBytes> + '_>;

    /// Gets a single value by key.
    fn get_pinned_cf(
        &self,
        cf: &impl AsColumnFamilyRef,
        key: impl AsRef<[u8]>,
    ) -> Option<DBPinnableSlice>;

    /// Gets multiple values by keys.
    ///
    /// Syntax note:
    /// The `<'a>` here is not special at all: it could technically be 100% inferred. Just the
    /// compiler feature allowing to skip it from within the `<Item = &...>` is not yet stable.
    /// TODO(when the rustc feature mentioned above becomes stable): get rid of the `<'a>`.
    fn multi_get_cf<'a>(
        &'a self,
        keys: impl IntoIterator<Item=(&'a (impl AsColumnFamilyRef + 'a), impl AsRef<[u8]>)>,
    ) -> Vec<Option<Vec<u8>>>;
}

/// A write-supporting extension of the [`ReadableRocks`].
///
/// Naturally, it is expected that only a "direct" RocksDB instance can implement this one.
pub trait WriteableRocks: ReadableRocks {
    /// Atomically writes the given batch of updates.
    fn write(&self, batch: WriteBatch);

    /// Returns a snapshot of the current state.
    fn snapshot(&self) -> SnapshotRocks;
}

/// A [`ReadableRocks`] instance opened as secondary instance.
pub trait SecondaryRocks: ReadableRocks {
    /// Tries to catch up with the primary by reading as much as possible from the
    /// log files.
    fn try_catchup_with_primary(&self);
}

/// RocksDB checkpoint support.
pub trait CheckpointableRocks {
    fn create_checkpoint(&self, checkpoint_path: PathBuf) -> Result<(), rocksdb::Error>;
}

/// Direct RocksDB instance.
pub struct DirectRocks {
    pub db: DB,
}

impl ReadableRocks for DirectRocks {
    fn cf_handle(&self, name: &str) -> &ColumnFamily {
        self.db.cf_handle(name).expect(name)
    }

    fn iterator_cf(
        &self,
        cf: &impl AsColumnFamilyRef,
        mode: IteratorMode,
    ) -> Box<dyn Iterator<Item=KVBytes> + '_> {
        Box::new(
            self.db
                .iterator_cf(cf, mode)
                .map(|result| result.expect("reading from DB iterator")),
        )
    }

    fn get_pinned_cf(
        &self,
        cf: &impl AsColumnFamilyRef,
        key: impl AsRef<[u8]>,
    ) -> Option<DBPinnableSlice> {
        self.db.get_pinned_cf(cf, key).expect("DB get by key")
    }

    fn multi_get_cf<'a>(
        &'a self,
        keys: impl IntoIterator<Item=(&'a (impl AsColumnFamilyRef + 'a), impl AsRef<[u8]>)>,
    ) -> Vec<Option<Vec<u8>>> {
        self.db
            .multi_get_cf(keys)
            .into_iter()
            .map(|result| result.expect("batch DB get by key"))
            .collect()
    }
}

impl WriteableRocks for DirectRocks {
    fn write(&self, batch: WriteBatch) {
        self.db.write(batch).expect("DB write batch");
    }

    fn snapshot(&self) -> SnapshotRocks {
        SnapshotRocks {
            db: &self.db,
            snapshot: self.db.snapshot(),
        }
    }
}

impl SecondaryRocks for DirectRocks {
    fn try_catchup_with_primary(&self) {
        self.db
            .try_catch_up_with_primary()
            .expect("secondary DB catchup");
    }
}

impl CheckpointableRocks for DirectRocks {
    fn create_checkpoint(&self, checkpoint_path: PathBuf) -> Result<(), rocksdb::Error> {
        create_checkpoint(&self.db, checkpoint_path)
    }
}

impl<'db> CheckpointableRocks for SnapshotRocks<'db> {
    fn create_checkpoint(&self, checkpoint_path: PathBuf) -> Result<(), rocksdb::Error> {
        create_checkpoint(self.db, checkpoint_path)
    }
}

fn create_checkpoint(db: &DB, checkpoint_path: PathBuf) -> Result<(), rocksdb::Error> {
    let checkpoint = Checkpoint::new(db)?;
    checkpoint.create_checkpoint(checkpoint_path)?;
    Ok(())
}

/// Snapshot of RocksDB.
///
/// Implementation note:
/// The original [`DB`] reference is interestingly kept internally by the [`Snapshot`] as well.
/// However, we need direct access to it for the [`Self::cf_handle()`] reasons.
pub struct SnapshotRocks<'db> {
    db: &'db DB,
    snapshot: Snapshot<'db>,
}

impl<'db> ReadableRocks for SnapshotRocks<'db> {
    fn cf_handle(&self, name: &str) -> &ColumnFamily {
        self.db.cf_handle(name).expect(name)
    }

    fn iterator_cf(
        &self,
        cf: &impl AsColumnFamilyRef,
        mode: IteratorMode,
    ) -> Box<dyn Iterator<Item=KVBytes> + '_> {
        Box::new(
            self.snapshot
                .iterator_cf(cf, mode)
                .map(|result| result.expect("reading from snapshot DB iterator")),
        )
    }

    fn get_pinned_cf(
        &self,
        cf: &impl AsColumnFamilyRef,
        key: impl AsRef<[u8]>,
    ) -> Option<DBPinnableSlice> {
        self.snapshot
            .get_pinned_cf(cf, key)
            .expect("snapshot DB get by key")
    }

    fn multi_get_cf<'a>(
        &'a self,
        keys: impl IntoIterator<Item=(&'a (impl AsColumnFamilyRef + 'a), impl AsRef<[u8]>)>,
    ) -> Vec<Option<Vec<u8>>> {
        self.snapshot
            .multi_get_cf(keys)
            .into_iter()
            .map(|result| result.expect("batch snapshot DB get by key"))
            .collect()
    }
}
