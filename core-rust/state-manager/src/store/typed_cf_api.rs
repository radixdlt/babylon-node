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

use radix_engine::types::*;
use rocksdb::{ColumnFamily, Direction, IteratorMode, WriteBatch, DB};

/// A higher-level database read/write context.
///
/// Operates with the following contract:
/// - All reads see the current DB state;
/// - All writes are accumulated in the internal buffer and are not visible to subsequent reads (of
///   this or other contexts), until [`TypedDbContext::flush()`] (either an explicit one, or an
///   implicit on [`Drop`]).
pub struct TypedDbContext<'db> {
    db: &'db DB,
    write_buffer: WriteBuffer,
}

impl<'db> TypedDbContext<'db> {
    /// Creates a new context, with an empty write buffer.
    pub fn new(db: &'db DB) -> Self {
        Self {
            db,
            write_buffer: WriteBuffer::default(),
        }
    }

    /// Returns a typed helper scoped at the given column family.
    pub fn cf<CF: TypedCf>(&self, cf: CF) -> TypedCfApi<'db, '_, CF> {
        TypedCfApi::new(self.db, cf, &self.write_buffer)
    }

    /// Explicitly flushes the current contents of the write buffer (so that it is visible to
    /// subsequent reads).
    pub fn flush(&self) {
        let write_batch = self.write_buffer.flip();
        if !write_batch.is_empty() {
            self.db.write(write_batch).expect("DB write batch");
        }
    }
}

impl<'db> Drop for TypedDbContext<'db> {
    fn drop(&mut self) {
        self.flush();
    }
}

/// A higher-level DB access API bound to its [`TypedDbContext`] and scoped at a specific column
/// family.
pub struct TypedCfApi<'db, 'wb, CF: TypedCf> {
    db: &'db DB,
    typed_cf: CF,
    write_buffer: &'wb WriteBuffer,
    cf_handle: &'db ColumnFamily, // only a cache - computable from `typed_cf`
    key_codec: CF::KeyCodec,      // only a cache - computable from `typed_cf`
    value_codec: CF::ValueCodec,  // only a cache - computable from `typed_cf`
}

impl<'db, 'wb, CF: TypedCf> TypedCfApi<'db, 'wb, CF> {
    /// Creates an instance for the given column family.
    fn new(db: &'db DB, typed_cf: CF, write_buffer: &'wb WriteBuffer) -> Self {
        // cache a few values:
        let cf_handle = db.cf_handle(CF::NAME).unwrap();
        let key_codec = typed_cf.key_codec();
        let value_codec = typed_cf.value_codec();
        Self {
            db,
            typed_cf,
            write_buffer,
            cf_handle,
            key_codec,
            value_codec,
        }
    }

    /// Gets value by key.
    pub fn get(&self, key: &CF::Key) -> Option<CF::Value> {
        self.db
            .get_pinned_cf(self.cf_handle, self.key_codec.encode(key).as_slice())
            .expect("database get by key")
            .map(|pinnable_slice| self.value_codec.decode(pinnable_slice.as_ref()))
    }

    /// Gets multiple values by keys.
    /// The order of returned values (or [`None`]s) matches the order of requested keys.
    pub fn get_many(&self, keys: Vec<&CF::Key>) -> Vec<Option<CF::Value>> {
        self.db
            .multi_get_cf(
                keys.into_iter()
                    .map(|key| (self.cf_handle, self.key_codec.encode(key))),
            )
            .into_iter()
            .map(|result| {
                result
                    .expect("multi get")
                    .map(|bytes| self.value_codec.decode(&bytes))
            })
            .collect()
    }

    /// Gets the entry of the least key _(according to the database's ordering)_.
    pub fn get_first(&self) -> Option<(CF::Key, CF::Value)> {
        self.iterate(Direction::Forward).next()
    }

    /// Gets the value associated with the least key _(according to the database's ordering)_.
    pub fn get_first_value(&self) -> Option<CF::Value> {
        self.get_first().map(|(_, value)| value)
    }

    /// Gets the entry of the greatest key _(according to the database's ordering)_.
    pub fn get_last(&self) -> Option<(CF::Key, CF::Value)> {
        self.iterate(Direction::Reverse).next()
    }

    /// Gets the greatest key _(according to the database's ordering)_.
    pub fn get_last_key(&self) -> Option<CF::Key> {
        self.get_last().map(|(key, _)| key)
    }

    /// Gets the value associated with the greatest key _(according to the database's ordering)_.
    pub fn get_last_value(&self) -> Option<CF::Value> {
        self.get_last().map(|(_, value)| value)
    }

    /// Returns an iterator traversing over (potentially) all the entries, in the requested
    /// direction.
    pub fn iterate(
        &self,
        direction: Direction,
    ) -> Box<dyn Iterator<Item = (CF::Key, CF::Value)> + 'db>
    where
        <CF as TypedCf>::KeyCodec: 'db,
        <CF as TypedCf>::ValueCodec: 'db,
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
    ) -> Box<dyn Iterator<Item = (CF::Key, CF::Value)> + 'db>
    where
        <CF as TypedCf>::KeyCodec: 'db,
        <CF as TypedCf>::ValueCodec: 'db,
    {
        self.iterate_with_mode(IteratorMode::From(
            self.key_codec.encode(from).as_slice(),
            direction,
        ))
    }

    /// Upserts the new value at the given key.
    pub fn put(&self, key: &CF::Key, value: &CF::Value) {
        self.write_buffer.put(
            self.cf_handle,
            self.key_codec.encode(key),
            self.value_codec.encode(value),
        );
    }

    /// Deletes the entry of the given key.
    pub fn delete(&self, key: &CF::Key) {
        self.write_buffer
            .delete(self.cf_handle, self.key_codec.encode(key));
    }

    /// Deletes the entries from the given key range.
    /// Follows the classic convention of "from inclusive, to exclusive".
    pub fn delete_range(&self, from_key: &CF::Key, to_key: &CF::Key) {
        self.write_buffer.delete_range(
            self.cf_handle,
            self.key_codec.encode(from_key),
            self.key_codec.encode(to_key),
        );
    }

    /// Returns an iterator based on the [`IteratorMode`] (which already contains encoded key).
    ///
    /// This is an internal shared implementation detail for different iteration flavors.
    fn iterate_with_mode(
        &self,
        mode: IteratorMode,
    ) -> Box<dyn Iterator<Item = (CF::Key, CF::Value)> + 'db>
    where
        <CF as TypedCf>::KeyCodec: 'db,
        <CF as TypedCf>::ValueCodec: 'db,
    {
        // create dedicated instances; do not reference those cached by `&self` from returned value:
        let key_codec = self.typed_cf.key_codec();
        let value_codec = self.typed_cf.value_codec();
        Box::new(
            self.db
                .iterator_cf(self.cf_handle, mode)
                .map(move |result| {
                    let (key, value) = result.expect("starting iteration");
                    (
                        key_codec.decode(key.as_ref()),
                        value_codec.decode(value.as_ref()),
                    )
                }),
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
}
