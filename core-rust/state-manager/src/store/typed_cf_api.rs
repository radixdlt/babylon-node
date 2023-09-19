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

/// A higher-level database access API scoped at a specific column family.
pub trait TypedCfApi<'db, K, V> {
    /// Gets value by key.
    fn get(&self, key: &K) -> Option<V>;

    /// Gets multiple values by keys.
    /// The order of returned values (or [`None`]s) matches the order of requested keys.
    fn get_many(&self, keys: Vec<&K>) -> Vec<Option<V>> {
        keys.into_iter().map(|key| self.get(key)).collect()
    }

    /// Gets the entry of the least key _(according to the database's ordering)_.
    fn get_first(&self) -> Option<(K, V)> {
        self.iterate(Direction::Forward).next()
    }

    /// Gets the least key _(according to the database's ordering)_.
    fn get_first_key(&self) -> Option<K> {
        self.get_first().map(|(key, _)| key)
    }

    /// Gets the value associated with the least key _(according to the database's ordering)_.
    fn get_first_value(&self) -> Option<V> {
        self.get_first().map(|(_, value)| value)
    }

    /// Gets the entry of the greatest key _(according to the database's ordering)_.
    fn get_last(&self) -> Option<(K, V)> {
        self.iterate(Direction::Reverse).next()
    }

    /// Gets the greatest key _(according to the database's ordering)_.
    fn get_last_key(&self) -> Option<K> {
        self.get_last().map(|(key, _)| key)
    }

    /// Gets the value associated with the greatest key _(according to the database's ordering)_.
    fn get_last_value(&self) -> Option<V> {
        self.get_last().map(|(_, value)| value)
    }

    /// Returns an iterator traversing over (potentially) all the entries, in the requested
    /// direction.
    fn iterate(&self, direction: Direction) -> Box<dyn Iterator<Item = (K, V)> + 'db>;

    /// Returns an iterator starting at the given key (inclusive) and traversing over (potentially)
    /// all the entries remaining in the requested direction.
    fn iterate_from(
        &self,
        from: &K,
        direction: Direction,
    ) -> Box<dyn Iterator<Item = (K, V)> + 'db>;

    /// Upserts the new value at the given key.
    fn put(&self, key: &K, value: &V);

    /// Adds the "upsert at key" operation to the given batch.
    fn put_with_batch(&self, batch: &mut WriteBatch, key: &K, value: &V);

    /// Adds the "delete by key" operation to the given batch.
    fn delete_with_batch(&self, batch: &mut WriteBatch, key: &K);

    /// Adds the "delete by key range" operation to the given batch.
    /// Follows the classic convention of "from inclusive, to exclusive".
    fn delete_range_with_batch(&self, batch: &mut WriteBatch, from_key: &K, to_key: &K);
}

/// An encoder/decoder of a typed value.
///
/// Design note:
/// There are reasons why this is a service-like business object (rather than requiring something
/// like `trait DbEncodable` to be implemented by types stored in the database):
/// - codecs are composable (e.g. `VersioningCodec::new(SborCodec::<MyType>::new())`);
/// - the same type may have different encodings (e.g. when used for a key vs for a value).
pub trait DbCodec<T>: Clone {
    /// Encodes the value into bytes.
    fn encode(&self, value: &T) -> Vec<u8>;
    /// Decodes the bytes into value.
    fn decode(&self, bytes: &[u8]) -> T;
}

/// A [`DB`]-backed implementation of [`TypedCfApi`] using configured key and value codecs.
pub struct CodecBasedCfApi<'db, K, KC: DbCodec<K> + 'db, V, VC: DbCodec<V> + 'db> {
    db: &'db DB,
    cf: &'db ColumnFamily,
    key_codec: KC,
    value_codec: VC,
    type_parameters_phantom: PhantomData<(K, V)>,
}

impl<'db, K, KC: DbCodec<K> + 'db, V, VC: DbCodec<V> + 'db> CodecBasedCfApi<'db, K, KC, V, VC> {
    /// Creates an instance for the given column family.
    pub fn new(db: &'db DB, cf_name: &str, key_codec: KC, value_codec: VC) -> Self {
        Self {
            db,
            cf: db.cf_handle(cf_name).unwrap(),
            key_codec,
            value_codec,
            type_parameters_phantom: PhantomData,
        }
    }

    /// Returns an iterator based on the [`IteratorMode`] (which already contains encoded key).
    ///
    /// This is an internal shared implementation detail for different iteration flavors.
    /// Implementation note: the key and value codecs are cloned, so that the iterator does not
    /// have to reference this instance of [`CodecBasedCfApi`] (for borrow-checker's reasons). This
    /// clone typically uses 0 bytes, though (in practice, iterators are stateless and have no
    /// dependencies).
    fn iterate_with_mode(&self, mode: IteratorMode) -> Box<dyn Iterator<Item = (K, V)> + 'db> {
        let key_codec = self.key_codec.clone();
        let value_codec = self.value_codec.clone();
        Box::new(self.db.iterator_cf(self.cf, mode).map(move |result| {
            let (key, value) = result.expect("starting iteration");
            (
                key_codec.decode(key.as_ref()),
                value_codec.decode(value.as_ref()),
            )
        }))
    }
}

impl<'db, K, KC: DbCodec<K> + 'db, V, VC: DbCodec<V> + 'db> TypedCfApi<'db, K, V>
    for CodecBasedCfApi<'db, K, KC, V, VC>
{
    fn get(&self, key: &K) -> Option<V> {
        self.db
            .get_pinned_cf(self.cf, self.key_codec.encode(key).as_slice())
            .expect("database get by key")
            .map(|pinnable_slice| self.value_codec.decode(pinnable_slice.as_ref()))
    }

    fn get_many(&self, keys: Vec<&K>) -> Vec<Option<V>> {
        self.db
            .multi_get_cf(
                keys.into_iter()
                    .map(|key| (self.cf, self.key_codec.encode(key))),
            )
            .into_iter()
            .map(|result| {
                result
                    .expect("multi get")
                    .map(|bytes| self.value_codec.decode(&bytes))
            })
            .collect()
    }

    fn iterate(&self, direction: Direction) -> Box<dyn Iterator<Item = (K, V)> + 'db> {
        self.iterate_with_mode(match direction {
            Direction::Forward => IteratorMode::Start,
            Direction::Reverse => IteratorMode::End,
        })
    }

    fn iterate_from(
        &self,
        from: &K,
        direction: Direction,
    ) -> Box<dyn Iterator<Item = (K, V)> + 'db> {
        self.iterate_with_mode(IteratorMode::From(
            self.key_codec.encode(from).as_slice(),
            direction,
        ))
    }

    fn put(&self, key: &K, value: &V) {
        self.db
            .put_cf(
                self.cf,
                self.key_codec.encode(key),
                self.value_codec.encode(value),
            )
            .expect("database put");
    }

    fn put_with_batch(&self, batch: &mut WriteBatch, key: &K, value: &V) {
        batch.put_cf(
            self.cf,
            self.key_codec.encode(key),
            self.value_codec.encode(value),
        );
    }

    fn delete_with_batch(&self, batch: &mut WriteBatch, key: &K) {
        batch.delete_cf(self.cf, self.key_codec.encode(key));
    }

    fn delete_range_with_batch(&self, batch: &mut WriteBatch, from_key: &K, to_key: &K) {
        batch.delete_range_cf(
            self.cf,
            self.key_codec.encode(from_key),
            self.key_codec.encode(to_key),
        );
    }
}

/// A trait for a type representing a specific version of some versioned type.
pub trait IsConcreteVersion {
    /// The type of versioned wrapper.
    type Versioned;

    /// Creates a versioned wrapper containing a copy of this instance.
    fn clone_into_versioned(&self) -> Self::Versioned;
}

/// A reusable versioning decorator for [`DbCodec`]s.
pub struct VersionedDbCodec<
    T: IsConcreteVersion<Versioned = VT>,
    U: DbCodec<VT>,
    VT: HasLatestVersion<Latest = T>,
> {
    underlying: U,
    type_parameters_phantom: PhantomData<VT>,
}

impl<T: IsConcreteVersion<Versioned = VT>, U: DbCodec<VT>, VT: HasLatestVersion<Latest = T>>
    VersionedDbCodec<T, U, VT>
{
    /// Applies versioning for the given codec.
    pub fn new(underlying: U) -> Self {
        Self {
            underlying,
            type_parameters_phantom: PhantomData,
        }
    }
}

impl<T: IsConcreteVersion<Versioned = VT>, U: DbCodec<VT>, VT: HasLatestVersion<Latest = T>> Clone
    for VersionedDbCodec<T, U, VT>
{
    fn clone(&self) -> Self {
        Self {
            underlying: self.underlying.clone(),
            type_parameters_phantom: PhantomData,
        }
    }
}

impl<T: IsConcreteVersion<Versioned = VT>, U: DbCodec<VT>, VT: HasLatestVersion<Latest = T>>
    DbCodec<T> for VersionedDbCodec<T, U, VT>
{
    fn encode(&self, value: &T) -> Vec<u8> {
        let versioned = value.clone_into_versioned();
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

impl<T: ScryptoEncode + ScryptoDecode> Clone for SborDbCodec<T> {
    fn clone(&self) -> Self {
        Self::default()
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

/// A [`DbCodec]` based on a predefined set of mappings.
#[derive(Clone, Default)]
pub struct PredefinedDbCodec<T: core::hash::Hash + Eq + Clone> {
    encoding: NonIterMap<T, Vec<u8>>,
    decoding: NonIterMap<Vec<u8>, T>,
}

impl PredefinedDbCodec<()> {
    /// Creates an instance capable of representing only a unit `()` (as an empty array).
    /// This is useful e.g. for "single-row" column families (which do not need keys), or "key-only"
    /// column families (which do not need values).
    pub fn for_unit() -> Self {
        Self::new(vec![((), vec![])])
    }
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
