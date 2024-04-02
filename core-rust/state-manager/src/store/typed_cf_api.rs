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
use crate::store::rocks_db::{ReadableRocks, WriteableRocks};
use itertools::Itertools;
use rocksdb::{ColumnFamily, Direction, IteratorMode, WriteBatch};
use std::ops::Range;

/// An optional-write-enabling marker trait to be used with [`TypedDbContext`].
///
/// This trait is `pub` only so that the callers of [`TypedDbContext`] can properly reference it.
/// It should not be implemented by any structs other than internally known options below (because
/// the [`TypedDbContext`] has to be statically aware of them - we selectively expose the available
/// write methods based on that).
pub trait WriteSupport {}

/// No write support.
pub struct NoWriteSupport;

impl WriteSupport for NoWriteSupport {}

/// Buffered write support.
///
/// All writes are accumulated in the internal buffer and are not visible to any subsequent reads,
/// until [`BufferedWriteSupport::flush()`] happens (either an explicit one, likely propagated from
/// [`TypedDbContext::flush()`], or an implicit one on [`Drop`]).
pub struct BufferedWriteSupport<'r, R: WriteableRocks> {
    buffer: WriteBuffer,
    rocks: &'r R,
}

impl<'r, R: WriteableRocks> BufferedWriteSupport<'r, R> {
    /// Creates an instance that will flush to the given RocksDB.
    pub fn new(rocks: &'r R) -> Self {
        Self {
            buffer: WriteBuffer::default(),
            rocks,
        }
    }
}

impl<'r, R: WriteableRocks> WriteSupport for BufferedWriteSupport<'r, R> {}

impl<'r, R: WriteableRocks> BufferedWriteSupport<'r, R> {
    /// Writes the batch to the RocksDB and flips the internal buffer.
    fn flush(&self) {
        let write_batch = self.buffer.flip();
        if !write_batch.is_empty() {
            self.rocks.write(write_batch);
        }
    }

    /// Returns the size of buffered data, in bytes.
    fn buffered_data_size(&self) -> usize {
        self.buffer.byte_size()
    }
}

impl<'r, R: WriteableRocks> Drop for BufferedWriteSupport<'r, R> {
    fn drop(&mut self) {
        self.flush();
    }
}

/// A higher-level database context.
///
/// All reads see the current DB state.
/// All (optional) write capabilities depend upon the used [`WriteSupport`].
pub struct TypedDbContext<'r, R: ReadableRocks, W: WriteSupport> {
    rocks: &'r R,
    write_support: W,
}

impl<'r, R: ReadableRocks, W: WriteSupport> TypedDbContext<'r, R, W> {
    /// Creates an instance using the given RocksDB.
    /// The write capabilities depend on the given [`WriteSupport`] implementation.
    pub fn new(rocks: &'r R, write_support: W) -> Self {
        Self {
            rocks,
            write_support,
        }
    }
}

impl<'r, R: WriteableRocks> TypedDbContext<'r, R, BufferedWriteSupport<'r, R>> {
    /// Explicitly flushes the current contents of the write buffer (so that it is visible to
    /// subsequent reads).
    pub fn flush(&self) {
        self.write_support.flush();
    }

    /// Returns the size of buffered data, in bytes.
    pub fn buffered_data_size(&self) -> usize {
        self.write_support.buffered_data_size()
    }
}

impl<'r, R: ReadableRocks, W: WriteSupport> TypedDbContext<'r, R, W> {
    /// Returns a typed helper scoped at the given column family.
    pub fn cf<CF: TypedCf>(&self, typed_cf: CF) -> TypedCfApi<'r, '_, CF, R, W> {
        TypedCfApi::new(
            ResolvedCf::resolve(self.rocks, typed_cf),
            self.rocks,
            &self.write_support,
        )
    }
}

/// A higher-level DB access API bound to its [`TypedDbContext`] and scoped at a specific column
/// family.
pub struct TypedCfApi<'r, 'w, CF: TypedCf, R: ReadableRocks, W: WriteSupport> {
    cf: ResolvedCf<'r, CF>,
    rocks: &'r R,
    write_support: &'w W,
}

impl<'r, 'w, CF: TypedCf, R: ReadableRocks, W: WriteSupport> TypedCfApi<'r, 'w, CF, R, W> {
    /// Creates an instance for the given column family.
    fn new(cf: ResolvedCf<'r, CF>, rocks: &'r R, write_support: &'w W) -> Self {
        Self {
            cf,
            rocks,
            write_support,
        }
    }

    /// Gets value by key.
    pub fn get(&self, key: &CF::Key) -> Option<CF::Value> {
        self.rocks
            .get_pinned_cf(self.cf.handle, self.cf.key_codec.encode(key).as_slice())
            .map(|pinnable_slice| self.cf.value_codec.decode(pinnable_slice.as_ref()))
    }

    /// Gets multiple values by keys.
    /// The order of returned values (or [`None`]s) matches the order of requested keys.
    pub fn get_many(&self, keys: Vec<&CF::Key>) -> Vec<Option<CF::Value>> {
        self.rocks
            .multi_get_cf(
                keys.into_iter()
                    .map(|key| (self.cf.handle, self.cf.key_codec.encode(key))),
            )
            .into_iter()
            .map(|result| result.map(|bytes| self.cf.value_codec.decode(&bytes)))
            .collect()
    }
}

impl<'r, 'w, CF: TypedCf, R: WriteableRocks>
    TypedCfApi<'r, 'w, CF, R, BufferedWriteSupport<'r, R>>
{
    /// Upserts the new value at the given key.
    pub fn put(&self, key: &CF::Key, value: &CF::Value) {
        self.write_support.buffer.put(
            self.cf.handle,
            self.cf.key_codec.encode(key),
            self.cf.value_codec.encode(value),
        );
    }

    /// Deletes the entry of the given key.
    pub fn delete(&self, key: &CF::Key) {
        self.write_support
            .buffer
            .delete(self.cf.handle, self.cf.key_codec.encode(key));
    }
}

impl<'r, 'w, KC: BoundedDbCodec, CF: TypedCf<KeyCodec = KC>, R: WriteableRocks>
    TypedCfApi<'r, 'w, CF, R, BufferedWriteSupport<'r, R>>
{
    /// Deletes all entries.
    pub fn delete_all(&self) {
        self.write_support.buffer.delete_range(
            self.cf.handle,
            vec![],
            self.cf.key_codec.upper_bound_encoding(),
        );
    }
}

impl<'r, 'w, KC: GroupPreservingDbCodec, CF: TypedCf<KeyCodec = KC>, R: WriteableRocks>
    TypedCfApi<'r, 'w, CF, R, BufferedWriteSupport<'r, R>>
{
    /// Deletes all the entries from the given group.
    pub fn delete_group(&self, group: &KC::Group) {
        let prefix_range = self.cf.key_codec.encode_group_range(group);
        self.write_support.buffer.delete_range(
            self.cf.handle,
            prefix_range.start,
            prefix_range.end,
        );
    }
}

impl<
        'r,
        'w,
        K,
        KC: OrderPreservingDbCodec + DbCodec<K>,
        CF: TypedCf<Key = K, KeyCodec = KC>,
        R: ReadableRocks,
        W: WriteSupport,
    > TypedCfApi<'r, 'w, CF, R, W>
{
    /// Gets the entry of the least key.
    pub fn get_first(&self) -> Option<(CF::Key, CF::Value)> {
        self.iterate(Direction::Forward).next()
    }

    /// Gets the least key.
    pub fn get_first_key(&self) -> Option<CF::Key> {
        self.get_first().map(|(key, _)| key)
    }

    /// Gets the value associated with the least key.
    pub fn get_first_value(&self) -> Option<CF::Value> {
        self.get_first().map(|(_, value)| value)
    }

    /// Gets the entry of the greatest key.
    pub fn get_last(&self) -> Option<(CF::Key, CF::Value)> {
        self.iterate(Direction::Reverse).next()
    }

    /// Gets the greatest key.
    pub fn get_last_key(&self) -> Option<CF::Key> {
        self.get_last().map(|(key, _)| key)
    }

    /// Gets the value associated with the greatest key.
    pub fn get_last_value(&self) -> Option<CF::Value> {
        self.get_last().map(|(_, value)| value)
    }

    /// Returns an iterator traversing over (potentially) all the entries, in the requested
    /// direction.
    pub fn iterate(
        &self,
        direction: Direction,
    ) -> Box<dyn Iterator<Item = (CF::Key, CF::Value)> + 'r>
    where
        CF::KeyCodec: 'r,
        CF::ValueCodec: 'r,
    {
        self.iterate_with_mode(match direction {
            Direction::Forward => IteratorMode::Start,
            Direction::Reverse => IteratorMode::End,
        })
    }

    /// Returns an iterator starting at the given key (inclusive) and traversing over (potentially)
    /// all the entries remaining in the requested direction.
    pub fn iterate_from(
        &self,
        from: &CF::Key,
        direction: Direction,
    ) -> Box<dyn Iterator<Item = (CF::Key, CF::Value)> + 'r>
    where
        CF::KeyCodec: 'r,
        CF::ValueCodec: 'r,
    {
        self.iterate_with_mode(IteratorMode::From(
            self.cf.key_codec.encode(from).as_slice(),
            direction,
        ))
    }

    /// Returns an iterator based on the [`IteratorMode`] (which already contains encoded key).
    ///
    /// This is an internal shared implementation detail for different iteration flavors.
    fn iterate_with_mode(
        &self,
        mode: IteratorMode,
    ) -> Box<dyn Iterator<Item = (CF::Key, CF::Value)> + 'r>
    where
        CF::KeyCodec: 'r,
        CF::ValueCodec: 'r,
    {
        // create dedicated instances; do not reference those cached by `&self` from returned value:
        let key_codec = self.cf.inner.key_codec();
        let value_codec = self.cf.inner.value_codec();
        Box::new(
            self.rocks
                .iterator_cf(self.cf.handle, mode)
                .map(move |(key, value)| {
                    (
                        key_codec.decode(key.as_ref()),
                        value_codec.decode(value.as_ref()),
                    )
                }),
        )
    }
}

impl<
        'r,
        'w,
        K,
        KC: OrderPreservingDbCodec + DbCodec<K>,
        CF: TypedCf<Key = K, KeyCodec = KC>,
        R: WriteableRocks,
    > TypedCfApi<'r, 'w, CF, R, BufferedWriteSupport<'r, R>>
{
    /// Deletes all the entries from the given key range.
    /// Follows the classic convention of "from inclusive, to exclusive".
    pub fn delete_range(&self, from_key: &CF::Key, to_key: &CF::Key) {
        self.write_support.buffer.delete_range(
            self.cf.handle,
            self.cf.key_codec.encode(from_key),
            self.cf.key_codec.encode(to_key),
        );
    }
}

impl<
        'r,
        'w,
        K,
        KC: IntraGroupOrderPreservingDbCodec<K> + DbCodec<K>,
        CF: TypedCf<Key = K, KeyCodec = KC>,
        R: ReadableRocks,
        W: WriteSupport,
    > TypedCfApi<'r, 'w, CF, R, W>
{
    /// Returns an iterator starting at the given key (inclusive) and traversing over (potentially)
    /// all the entries remaining *in this element's group*, in the requested direction.
    pub fn iterate_group_from(
        &self,
        from: &CF::Key,
        direction: Direction,
    ) -> Box<dyn Iterator<Item = (CF::Key, CF::Value)> + 'r>
    where
        CF::KeyCodec: 'r,
        CF::ValueCodec: 'r,
    {
        let group = self.cf.key_codec.resolve_group_of(from);
        let group_range = self.cf.key_codec.encode_group_range(&group);
        let key_codec = self.cf.inner.key_codec();
        let value_codec = self.cf.inner.value_codec();
        Box::new(
            self.rocks
                .iterator_cf(
                    self.cf.handle,
                    IteratorMode::From(&self.cf.key_codec.encode(from), direction),
                )
                .take_while(move |(key, _value)| match direction {
                    Direction::Forward => key.as_ref() < group_range.end.as_slice(),
                    Direction::Reverse => key.as_ref() >= group_range.start.as_slice(),
                })
                .map(move |(key, value)| {
                    (
                        key_codec.decode(key.as_ref()),
                        value_codec.decode(value.as_ref()),
                    )
                }),
        )
    }

    /// Returns an iterator over all groups (as defined by [`GroupPreservingDbCodec`]) of keys, in
    /// a deterministic but arbitrary order.
    ///
    /// *Performance note:*
    /// This method iterates over *all* entries, extracts keys' groups and deduplicates them. This
    /// involves a lot of "wasted" DB reads and thus makes it not suitable for production purposes
    /// (i.e. an index of groups should be used instead).
    /// Hence, this method is meant only for test / investigation / DB verification purposes.
    pub fn iterate_key_groups(&self) -> Box<dyn Iterator<Item = KC::Group> + 'r>
    where
        CF::KeyCodec: 'r,
        KC::Group: PartialEq,
    {
        let key_codec = self.cf.inner.key_codec();
        Box::new(
            self.rocks
                .iterator_cf(self.cf.handle, IteratorMode::Start)
                .map(move |(key_bytes, _)| {
                    let key = key_codec.decode(key_bytes.as_ref());
                    key_codec.resolve_group_of(&key)
                })
                // We have the group-preserving guarantee from our key codec, which means that all
                // elements of the same group will be next to each other when iterated
                // lexicographically from the DB. Hence, it is sufficient to remove *consecutive*
                // duplicates (i.e. as `dedup()` does).
                .dedup(),
        )
    }
}

/// A definition of a typed column family.
///
/// This is the most verbose and customizable trait. Usual cases can use one of the more convenient
/// traits defined below.
pub trait TypedCf {
    /// Type of the key.
    type Key;
    /// Type of the value.
    type Value;

    /// Type of the [`DbCodec`] for the keys.
    type KeyCodec: DbCodec<Self::Key>;

    /// Type of the [`DbCodec`] for the values.
    type ValueCodec: DbCodec<Self::Value>;

    /// Column family name (as known to the DB).
    const NAME: &'static str;
    /// Creates a new [`DbCodec`] for keys within this column family.
    fn key_codec(&self) -> Self::KeyCodec;
    /// Creates a new [`DbCodec`] for values within this column family.
    fn value_codec(&self) -> Self::ValueCodec;
}

/// A convenience trait implementing [`TypedCf`] for a simple case where both [`DbCodec`]s have
/// cheap [`Default`] implementations.
pub trait DefaultCf {
    /// Type of the key.
    type Key;
    /// Type of the value.
    type Value;

    /// Column family name (as known to the DB).
    ///
    /// Note: this deliberately uses a different identifier than [`TypedCf::NAME`] to avoid awkward
    /// fully-qualified syntax wherever it is used.
    const DEFAULT_NAME: &'static str;
    /// Key codec type.
    type KeyCodec: Default;
    /// Value codec type.
    type ValueCodec: Default;
}

impl<
        K,
        V,
        KC: Default + DbCodec<K>,
        VC: Default + DbCodec<V>,
        D: DefaultCf<Key = K, Value = V, KeyCodec = KC, ValueCodec = VC>,
    > TypedCf for D
{
    type Key = K;
    type Value = V;

    type KeyCodec = KC;
    type ValueCodec = VC;

    const NAME: &'static str = Self::DEFAULT_NAME;

    fn key_codec(&self) -> KC {
        KC::default()
    }

    fn value_codec(&self) -> VC {
        VC::default()
    }
}

/// A convenience trait implementing [`TypedCf`] for a popular case where a "versioned SBOR"
/// encoding is used for values.
pub trait VersionedCf {
    type Key;
    type Value;

    /// Column family name (as known to the DB).
    ///
    /// Note: this deliberately uses a different identifier than [`TypedCf::NAME`] to avoid awkward
    /// fully-qualified syntax wherever it is used.
    const VERSIONED_NAME: &'static str;
    /// Key codec type.
    type KeyCodec: Default;
    /// Versioned **value** type (a codec for it is known, i.e. SBOR-based).
    type VersionedValue;
}

impl<K, V, VV, KC, D> DefaultCf for D
where
    V: Into<VV> + Clone,
    VV: ScryptoEncode + ScryptoDecode + HasLatestVersion<Latest = V>,
    KC: Default,
    D: VersionedCf<Key = K, Value = V, KeyCodec = KC, VersionedValue = VV>,
{
    type Key = K;
    type Value = V;

    const DEFAULT_NAME: &'static str = Self::VERSIONED_NAME;
    type KeyCodec = KC;
    type ValueCodec = VersionedDbCodec<SborDbCodec<VV>, V, VV>;
}

/// An encoder/decoder of a typed value.
///
/// Design note:
/// There are reasons why this is a service-like business object (rather than requiring something
/// like `trait DbEncodable` to be implemented by types stored in the database):
/// - codecs are composable (e.g. `VersioningCodec::new(SborCodec::<MyType>::new())`);
/// - the same type may have different encodings (e.g. when used for a key vs for a value).
pub trait DbCodec<T> {
    /// Encodes the value into bytes.
    fn encode(&self, value: &T) -> Vec<u8>;
    /// Decodes the bytes into value.
    fn decode(&self, bytes: &[u8]) -> T;
}

/// An extra trait to be implemented on [`DbCodec`]s which know an upper bound on their subjects'
/// encoding.
///
/// This capability allows e.g. for an efficient, atomic "delete all" operation (using a range of
/// keys `[vec![], upper_bound)`).
pub trait BoundedDbCodec {
    /// Returns an encoding of an (exclusive) upper bound element.
    fn upper_bound_encoding(&self) -> Vec<u8>;
}

/// A marker trait which must only be implemented on [`DbCodec`]s which preserve the business-level
/// ordering of values when encoding/decoding.
///
/// More formally: Such codec must translate the natural ordering (i.e. [`Ord`]) of its `<T>` values
/// into a *lexicographical* ordering of their byte representations.
///
/// Examples:
/// - a `DbCodec<u32>` which turns an integer into 4 *big-endian* bytes *does* preserve ordering:
///   - `1u32` <-> `[0, 0, 0, 1]`,
///   - `7u32` <-> `[0, 0, 0, 7]`,
///   - `259u32` <-> `[0, 0, 1, 3]`,
///   - and so on: the left side increases naturally and the right side increases lexicographically.
/// - a `DbCodec<u32>` which turns an integer into ASCII string bytes *does not* preserve ordering:
///   - `1u32` <-> `[49]`,
///   - `7u32` <-> `[55]`,
///   - `259u32` <-> `[50, 53, 59]`,
///   - order broken: the right side *does not* consistently increase lexicographically (the bytes
///     starting with `[50, ...]` are lexicographically before `[55]`).
///
/// The order preservation is important for database *key* codecs of column families which need to
/// support e.g. iteration of elements starting from a particular element, or any batch operations
/// defined by `[from, to]` ranges.
pub trait OrderPreservingDbCodec {}

/// An extra trait to be implemented on [`DbCodec`]s which preserve the business-level grouping of
/// values when encoding/decoding.
///
/// More formally: if a set of values `<T>` all share the same [`Self::Group`], then their byte
/// representations must all share the same prefix (and vice versa).
///
/// Examples:
/// - a `DbCodec<SocketAddress>` which turns an `(ip: u32, port: u16)` tuple into `ip[4B]|port[2B]`
///   bytes *does* preserve grouping by host:
///   - [3, 14, 0, 1, 0, 80] and [3, 14, 0, 1, 0, 22] bytes start with the same 4-byte prefix, and
///    indeed they represent port 80 and port 22 on the same host `3.14.0.1`.
///   - the lexicographically-ordered range of *all* socket addresses on host `3.14.0.1` can be
///     expressed as "from `[3, 14, 0, 1]` inclusive to `[3, 14, 0, 1, 255, 255, 0]` exclusive"
///     (please note that it requires some knowledge on the maximum length of the part following the
///     prefix).
/// - a `DbCodec<Person>` which turns a `(first_name: String, last_name: String)` tuple into
///   `<first_name> <last_name>` ASCII strings *does not* preserve grouping by families:
///   - `("John", "Doe")` <-> `[J, o, h, n,  , D, o, e]`,
///   - `("Ann", "Doe")` <-> `[A, n, n,  , D, o, e]`,
///   - grouping broken: even though John and Ann belong to the same family, they *do not* share
///     any well-specified prefix in their byte representations.
///   - a lexicographically-ordered range covering all members of a family *cannot* be constructed
///     under this encoding.
///   - the grouping in this case *could* be preserved e.g. by encoding `<last_name> <first_name>`
///     (with a variable-length prefix, defined as "everything before the first space character").
///
/// The grouping preservation is important for database *key* codecs of column families which need
/// to support e.g. a batch delete operation of an entire group.
pub trait GroupPreservingDbCodec {
    type Group;

    /// Encodes the group into a [`Range`] of byte representations that covers all values belonging
    /// to that group.
    ///
    /// Please note that:
    /// - the returned range *must* cover at least *all valid* group members,
    /// - it *may* cover some inexistent/invalid/not-occurring-in-practice group members,
    /// - but it *must not* cover any member of any other group.
    fn encode_group_range(&self, group: &Self::Group) -> Range<Vec<u8>>;
}

/// An extra trait to be implemented on [`DbCodec`]s which preserve the business-level ordering of
/// values *within groups* (as defined by [`GroupPreservingDbCodec`]).
///
/// Intuitively: Such codec gives the [`OrderPreservingDbCodec`]'s guarantees *only* within each
/// range of keys belonging to the same group. And a group's keys are already consecutive, thanks to
/// the [`GroupPreservingDbCodec`] supertrait.
///
/// The group's order preservation is important for database *key* codecs of column families which
/// follow a classic "partition key + sort key" pattern.
pub trait IntraGroupOrderPreservingDbCodec<T>: GroupPreservingDbCodec {
    /// Determines the group which the given value belongs to.
    fn resolve_group_of(&self, value: &T) -> <Self as GroupPreservingDbCodec>::Group;
}

/// A reusable versioning decorator for [`DbCodec`]s.
pub struct VersionedDbCodec<U: DbCodec<VT>, T: Into<VT> + Clone, VT: HasLatestVersion<Latest = T>> {
    underlying: U,
    type_parameters_phantom: PhantomData<VT>,
}

impl<U: DbCodec<VT> + Default, T: Into<VT> + Clone, VT: HasLatestVersion<Latest = T>> Default
    for VersionedDbCodec<U, T, VT>
{
    fn default() -> Self {
        Self {
            underlying: U::default(),
            type_parameters_phantom: PhantomData,
        }
    }
}

impl<U: DbCodec<VT>, T: Into<VT> + Clone, VT: HasLatestVersion<Latest = T>> DbCodec<T>
    for VersionedDbCodec<U, T, VT>
{
    fn encode(&self, value: &T) -> Vec<u8> {
        let versioned = value.clone().into();
        self.underlying.encode(&versioned)
    }

    fn decode(&self, bytes: &[u8]) -> T {
        let versioned = self.underlying.decode(bytes);
        versioned.into_latest()
    }
}

/// A [`DbCodec]` for SBOR types.
pub struct SborDbCodec<T: ScryptoEncode + ScryptoDecode> {
    type_parameters_phantom: PhantomData<T>,
}

impl<T: ScryptoEncode + ScryptoDecode> Default for SborDbCodec<T> {
    fn default() -> Self {
        Self {
            type_parameters_phantom: PhantomData,
        }
    }
}

impl<T: ScryptoEncode + ScryptoDecode> DbCodec<T> for SborDbCodec<T> {
    fn encode(&self, value: &T) -> Vec<u8> {
        scrypto_encode(value).unwrap()
    }

    fn decode(&self, bytes: &[u8]) -> T {
        scrypto_decode(bytes).unwrap()
    }
}

/// A [`DbCodec]` for byte arrays (`Vec<u8>`) that are supposed to be stored directly.
#[derive(Clone, Default)]
pub struct DirectDbCodec {}

impl DbCodec<Vec<u8>> for DirectDbCodec {
    fn encode(&self, value: &Vec<u8>) -> Vec<u8> {
        value.clone()
    }

    fn decode(&self, bytes: &[u8]) -> Vec<u8> {
        bytes.to_vec()
    }
}

/// A [`DbCodec]` capable of representing only a unit `()` (as an empty array).
/// This is useful e.g. for "single-row" column families (which do not need keys), or "key-only"
/// column families (which do not need values).
#[derive(Clone, Default)]
pub struct UnitDbCodec {}

impl DbCodec<()> for UnitDbCodec {
    fn encode(&self, _value: &()) -> Vec<u8> {
        vec![]
    }

    fn decode(&self, bytes: &[u8]) {
        assert_eq!(bytes.len(), 0);
    }
}

/// A [`DbCodec]` based on a predefined set of mappings.
#[derive(Default)]
pub struct PredefinedDbCodec<T: core::hash::Hash + Eq + Clone> {
    encoding: NonIterMap<T, Vec<u8>>,
    decoding: NonIterMap<Vec<u8>, T>,
}

impl<T: core::hash::Hash + Eq + Clone> PredefinedDbCodec<T> {
    /// Creates an instance from the given `(value, encoding)` mapping pairs.
    pub fn new(mappings: impl IntoIterator<Item = (T, Vec<u8>)>) -> Self {
        let mut encoding = NonIterMap::new();
        let mut decoding = NonIterMap::new();
        for (value, bytes) in mappings {
            encoding.insert(value.clone(), bytes.clone());
            decoding.insert(bytes, value);
        }
        Self { encoding, decoding }
    }
}

impl<T: core::hash::Hash + Eq + Clone + ToString> PredefinedDbCodec<T> {
    /// Creates an instance mapping between the given values and their [`ToString`] representations.
    pub fn new_from_string_representations(values: impl IntoIterator<Item = T>) -> Self {
        Self::new(
            values
                .into_iter()
                .map(|value| (value.clone(), value.to_string().into_bytes())),
        )
    }
}

impl<T: core::hash::Hash + Eq + Clone> DbCodec<T> for PredefinedDbCodec<T> {
    fn encode(&self, value: &T) -> Vec<u8> {
        self.encoding
            .get(value)
            .expect("value outside mappings")
            .clone()
    }

    fn decode(&self, bytes: &[u8]) -> T {
        self.decoding
            .get(bytes)
            .expect("encoding outside mappings")
            .clone()
    }
}

/// An internal wrapper for [`WriteBatch`], allowing to use it with interior mutability.
#[derive(Default)]
struct WriteBuffer {
    write_batch: RefCell<WriteBatch>,
}

impl WriteBuffer {
    pub fn put(&self, cf: &ColumnFamily, key: Vec<u8>, value: Vec<u8>) {
        self.write_batch.borrow_mut().put_cf(cf, key, value);
    }

    pub fn delete(&self, cf: &ColumnFamily, key: Vec<u8>) {
        self.write_batch.borrow_mut().delete_cf(cf, key);
    }

    pub fn delete_range(&self, cf: &ColumnFamily, from: Vec<u8>, to: Vec<u8>) {
        self.write_batch.borrow_mut().delete_range_cf(cf, from, to);
    }

    pub fn flip(&self) -> WriteBatch {
        self.write_batch.replace(WriteBatch::default())
    }

    pub fn byte_size(&self) -> usize {
        self.write_batch.borrow().size_in_bytes()
    }
}

/// An internal wrapper for a [`TypedCf`] and dependencies resolved from it.
struct ResolvedCf<'r, CF: TypedCf> {
    inner: CF,
    handle: &'r ColumnFamily,    // only a cache - computable from `typed_cf`
    key_codec: CF::KeyCodec,     // only a cache - computable from `typed_cf`
    value_codec: CF::ValueCodec, // only a cache - computable from `typed_cf`
}

impl<'r, CF: TypedCf> ResolvedCf<'r, CF> {
    /// Resolves and caches properties of the given [`TypedCf`].
    pub fn resolve<R: ReadableRocks>(rocks: &'r R, inner: CF) -> Self {
        let handle = rocks.cf_handle(CF::NAME);
        let key_codec = inner.key_codec();
        let value_codec = inner.value_codec();
        Self {
            inner,
            handle,
            key_codec,
            value_codec,
        }
    }
}
