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

use std::ops::Range;

use crate::engine_prelude::*;

use crate::StateVersion;
use crate::store::traits::indices::CreationId;
use crate::store::traits::scenario::ScenarioSequenceNumber;
use crate::store::typed_cf_api::*;
use crate::transaction::RawLedgerTransaction;

#[derive(Default)]
pub struct StateVersionDbCodec {}

impl DbCodec<StateVersion> for StateVersionDbCodec {
    fn encode(&self, value: &StateVersion) -> Vec<u8> {
        value.to_be_bytes().to_vec()
    }

    fn decode(&self, bytes: &[u8]) -> StateVersion {
        StateVersion::from_be_bytes(bytes)
    }
}

impl OrderPreservingDbCodec for StateVersionDbCodec {}

#[derive(Default)]
pub struct EpochDbCodec {}

impl DbCodec<Epoch> for EpochDbCodec {
    fn encode(&self, value: &Epoch) -> Vec<u8> {
        value.number().to_be_bytes().to_vec()
    }

    fn decode(&self, bytes: &[u8]) -> Epoch {
        Epoch::of(u64::from_be_bytes(copy_u8_array(bytes)))
    }
}

impl OrderPreservingDbCodec for EpochDbCodec {}

#[derive(Default)]
pub struct ScenarioSequenceNumberDbCodec {}

impl DbCodec<ScenarioSequenceNumber> for ScenarioSequenceNumberDbCodec {
    fn encode(&self, value: &ScenarioSequenceNumber) -> Vec<u8> {
        value.to_be_bytes().to_vec()
    }

    fn decode(&self, bytes: &[u8]) -> ScenarioSequenceNumber {
        ScenarioSequenceNumber::from_be_bytes(copy_u8_array(bytes))
    }
}

impl OrderPreservingDbCodec for ScenarioSequenceNumberDbCodec {}

#[derive(Default)]
pub struct RawLedgerTransactionDbCodec {}

impl DbCodec<RawLedgerTransaction> for RawLedgerTransactionDbCodec {
    fn encode(&self, value: &RawLedgerTransaction) -> Vec<u8> {
        value.0.to_vec()
    }

    fn decode(&self, bytes: &[u8]) -> RawLedgerTransaction {
        RawLedgerTransaction(bytes.to_vec())
    }
}

pub struct HashDbCodec<T: IsHash> {
    type_parameters_phantom: PhantomData<T>,
}

impl<T: IsHash> Default for HashDbCodec<T> {
    fn default() -> Self {
        Self {
            type_parameters_phantom: PhantomData,
        }
    }
}

impl<T: IsHash> DbCodec<T> for HashDbCodec<T> {
    fn encode(&self, value: &T) -> Vec<u8> {
        value.as_slice().to_vec()
    }

    fn decode(&self, bytes: &[u8]) -> T {
        T::from_bytes(copy_u8_array(bytes))
    }
}

/// A codec for keys of substates within the [`SubstateDatabase`].
///
/// Implementation notes:
///
/// Each key is a tuple of `(node key [N bytes], partition number [1 byte], sort key [S bytes])`.
/// In practice, for our current database, the `N` above is of static length, but this codec can in
/// theory support variable lengths (by "length|value" encoding). The regrettable trade-off is that
/// the resulting byte representations do *not* preserve the lexicographical ordering (i.e. this
/// codec cannot implement [`OrderPreservingDbCodec`]). It is still a [`GroupPreservingDbCodec`]
/// (i.e. it knows the substates' grouping into partitions).
///
/// The resulting byte representation is:
///
/// `[N, node_key[0], node_key[1], ..., node_key[N-1], partition_num, sort_key[0], sort_key[1], ..., sort_key[S-1]]`
///
/// An example:
///
/// `(NodeKey[1, 3, 3, 7], PartitionNum(88), SortKey(200, 100, 150))`
/// is encoded into:
/// `[4, 1, 3, 3, 7, 88, 200, 100, 150]`
#[derive(Default)]
pub struct SubstateKeyDbCodec {}

impl DbCodec<DbSubstateKey> for SubstateKeyDbCodec {
    fn encode(&self, value: &DbSubstateKey) -> Vec<u8> {
        let (partition_key, sort_key) = value;
        let mut buffer =
            Vec::with_capacity(1 + partition_key.node_key.len() + 1 + sort_key.0.len());
        buffer.push(
            u8::try_from(partition_key.node_key.len())
                .expect("Node key length is effectively constant known to fit in u8"),
        );
        buffer.extend(partition_key.node_key.clone());
        buffer.push(partition_key.partition_num);
        buffer.extend(sort_key.0.clone());
        buffer
    }

    fn decode(&self, buffer: &[u8]) -> DbSubstateKey {
        let node_key_start: usize = 1usize;
        let partition_key_start = 1 + usize::from(buffer[0]);
        let sort_key_start = 1 + partition_key_start;

        let node_key = buffer[node_key_start..partition_key_start].to_vec();
        let partition_num = buffer[partition_key_start];
        let sort_key = buffer[sort_key_start..].to_vec();
        (
            DbPartitionKey {
                node_key,
                partition_num,
            },
            DbSortKey(sort_key),
        )
    }
}

impl GroupPreservingDbCodec for SubstateKeyDbCodec {
    type Group = DbPartitionKey;

    fn encode_group_range(&self, partition_key: &Self::Group) -> Range<Vec<u8>> {
        let full_db_sort_key_range = Self::full_db_sort_key_range();
        let from = self.encode(&(partition_key.clone(), full_db_sort_key_range.start));
        let to = self.encode(&(partition_key.clone(), full_db_sort_key_range.end));
        from..to
    }
}

impl IntraGroupOrderPreservingDbCodec<DbSubstateKey> for SubstateKeyDbCodec {
    fn resolve_group_of(&self, key: &DbSubstateKey) -> <Self as GroupPreservingDbCodec>::Group {
        key.0.clone()
    }
}

impl SubstateKeyDbCodec {
    /// Returns a [`Range`] guaranteed to cover all allowed [`DbSortKey`]s.
    ///
    /// Implementation note:
    ///
    /// The lower bound is a trivial empty key (as for every lexicographically ordered type).
    /// The upper bound relies on the [`MAX_SUBSTATE_KEY_SIZE`] defined by the Engine. For extra
    /// safety, we double the length (Engine reportedly adds some minor, static overhead for its
    /// internal purposes).
    ///
    /// This means that the returned [`Range`] *also* contains a lot of [`DbSortKey`]s which would
    /// *not* be allowed by the Engine. Thus, it is *not* suitable e.g. for input key validation.
    /// Its primary use-case is for deleting a range of keys representing an entire partition.
    pub fn full_db_sort_key_range() -> Range<DbSortKey> {
        DbSortKey(vec![])..DbSortKey(vec![u8::MAX; 2 * MAX_SUBSTATE_KEY_SIZE])
    }
}

#[derive(Default)]
pub struct NodeKeyDbCodec {}

impl DbCodec<NodeKey> for NodeKeyDbCodec {
    fn encode(&self, value: &NodeKey) -> Vec<u8> {
        encode_key(value)
    }

    fn decode(&self, _bytes: &[u8]) -> NodeKey {
        unimplemented!("no use-case for decoding hash tree's `NodeKey`s exists yet")
    }
}

pub struct PrefixGlobalAddressDbCodec<S, SC: DbCodec<S>> {
    suffix_codec: SC,
    type_parameters_phantom: PhantomData<S>,
}

impl<S, SC: DbCodec<S>> PrefixGlobalAddressDbCodec<S, SC> {
    pub fn new(suffix_codec: SC) -> Self {
        Self {
            suffix_codec,
            type_parameters_phantom: PhantomData,
        }
    }
}

impl<S, SC: DbCodec<S>> DbCodec<(GlobalAddress, S)> for PrefixGlobalAddressDbCodec<S, SC> {
    fn encode(&self, (global_address, suffix): &(GlobalAddress, S)) -> Vec<u8> {
        let mut encoding = global_address.to_vec();
        encoding.extend_from_slice(self.suffix_codec.encode(suffix).as_slice());
        encoding
    }

    fn decode(&self, bytes: &[u8]) -> (GlobalAddress, S) {
        let global_address = GlobalAddress::new_or_panic(copy_u8_array(&bytes[..NodeId::LENGTH]));
        let suffix = self.suffix_codec.decode(&bytes[NodeId::LENGTH..]);
        (global_address, suffix)
    }
}

impl<S, SC: DbCodec<S> + OrderPreservingDbCodec> OrderPreservingDbCodec
    for PrefixGlobalAddressDbCodec<S, SC>
{
}

#[derive(Default)]
pub struct NodeIdDbCodec {}

impl DbCodec<NodeId> for NodeIdDbCodec {
    fn encode(&self, value: &NodeId) -> Vec<u8> {
        value.0.to_vec()
    }

    fn decode(&self, bytes: &[u8]) -> NodeId {
        NodeId(copy_u8_array(bytes))
    }
}

#[derive(Default)]
pub struct TypeAndCreationIndexKeyDbCodec {}

impl DbCodec<(EntityType, CreationId)> for TypeAndCreationIndexKeyDbCodec {
    fn encode(&self, value: &(EntityType, CreationId)) -> Vec<u8> {
        let (
            entity_type,
            CreationId {
                state_version,
                index_within_txn,
            },
        ) = value;
        let mut bytes = Vec::new();
        bytes.push(*entity_type as u8);
        bytes.extend_from_slice(&state_version.to_be_bytes());
        bytes.extend_from_slice(&index_within_txn.to_be_bytes());
        bytes
    }

    fn decode(&self, bytes: &[u8]) -> (EntityType, CreationId) {
        let (entity_type_byte, bytes) = bytes.split_at(1);
        let entity_type = EntityType::from_repr(entity_type_byte[0]).expect("unexpected type byte");

        let (state_version_bytes, index_within_txn_bytes) = bytes.split_at(StateVersion::BYTE_LEN);
        let state_version = StateVersion::from_be_bytes(state_version_bytes);
        let index_within_txn = u32::from_be_bytes(copy_u8_array(index_within_txn_bytes));

        (
            entity_type,
            CreationId {
                state_version,
                index_within_txn,
            },
        )
    }
}

impl GroupPreservingDbCodec for TypeAndCreationIndexKeyDbCodec {
    type Group = EntityType;

    fn encode_group_range(&self, entity_type: &EntityType) -> Range<Vec<u8>> {
        let Range { start, end } = CreationId::full_range();
        Range {
            start: self.encode(&(*entity_type, start)),
            end: self.encode(&(*entity_type, end)),
        }
    }
}

impl IntraGroupOrderPreservingDbCodec<(EntityType, CreationId)> for TypeAndCreationIndexKeyDbCodec {
    fn resolve_group_of(&self, value: &(EntityType, CreationId)) -> EntityType {
        let (entity_type, _) = value;
        *entity_type
    }
}

#[derive(Default)]
pub struct BlueprintAndCreationIndexKeyDbCodec {}

impl DbCodec<(PackageAddress, Hash, CreationId)> for BlueprintAndCreationIndexKeyDbCodec {
    fn encode(&self, value: &(PackageAddress, Hash, CreationId)) -> Vec<u8> {
        let (
            package_address,
            blueprint_name_hash,
            CreationId {
                state_version,
                index_within_txn,
            },
        ) = value;
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&package_address.as_node_id().0);
        bytes.extend_from_slice(&blueprint_name_hash.0);
        bytes.extend_from_slice(&state_version.to_be_bytes());
        bytes.extend_from_slice(&index_within_txn.to_be_bytes());
        bytes
    }

    fn decode(&self, bytes: &[u8]) -> (PackageAddress, Hash, CreationId) {
        let (package_address_bytes, bytes) = bytes.split_at(NodeId::LENGTH);
        let package_address =
            PackageAddress::try_from(package_address_bytes).expect("invalid package address");

        let (blueprint_name_hash_bytes, bytes) = bytes.split_at(Hash::LENGTH);
        let blueprint_name_hash = Hash::from_bytes(copy_u8_array(blueprint_name_hash_bytes));

        let (state_version_bytes, index_within_txn_bytes) = bytes.split_at(StateVersion::BYTE_LEN);
        let state_version = StateVersion::from_be_bytes(state_version_bytes);
        let index_within_txn = u32::from_be_bytes(copy_u8_array(index_within_txn_bytes));

        (
            package_address,
            blueprint_name_hash,
            CreationId {
                state_version,
                index_within_txn,
            },
        )
    }
}

impl GroupPreservingDbCodec for BlueprintAndCreationIndexKeyDbCodec {
    type Group = (PackageAddress, Hash);

    fn encode_group_range(&self, group: &(PackageAddress, Hash)) -> Range<Vec<u8>> {
        let (package_address, blueprint_name_hash) = group;
        let Range { start, end } = CreationId::full_range();
        Range {
            start: self.encode(&(*package_address, *blueprint_name_hash, start)),
            end: self.encode(&(*package_address, *blueprint_name_hash, end)),
        }
    }
}

impl IntraGroupOrderPreservingDbCodec<(PackageAddress, Hash, CreationId)>
    for BlueprintAndCreationIndexKeyDbCodec
{
    fn resolve_group_of(
        &self,
        value: &(PackageAddress, Hash, CreationId),
    ) -> (PackageAddress, Hash) {
        let (package_address, blueprint_name_hash, _) = value;
        (*package_address, *blueprint_name_hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rocksdb_key_encoding_is_invertible() {
        let codec = SubstateKeyDbCodec {};

        let partition_key = DbPartitionKey {
            node_key: vec![1, 2, 3, 4, 132],
            partition_num: 224,
        };
        let sort_key = DbSortKey(vec![13, 5]);
        let buffer = codec.encode(&(partition_key.clone(), sort_key.clone()));

        let decoded = codec.decode(&buffer);

        assert_eq!(partition_key, decoded.0);
        assert_eq!(sort_key, decoded.1);
    }

    #[test]
    fn rocksdb_key_encoding_respects_lexicographic_ordering_on_sort_keys() {
        let codec = SubstateKeyDbCodec::default();

        let partition_key = DbPartitionKey {
            node_key: vec![73, 85],
            partition_num: 1,
        };
        let sort_key = DbSortKey(vec![0, 4]);
        let iterator_start = codec.encode(&(partition_key.clone(), sort_key.clone()));

        assert!(codec.encode(&(partition_key.clone(), DbSortKey(vec![0]))) < iterator_start);
        assert!(codec.encode(&(partition_key.clone(), DbSortKey(vec![0, 3]))) < iterator_start);
        assert_eq!(
            codec.encode(&(partition_key.clone(), DbSortKey(vec![0, 4]))),
            iterator_start
        );
        assert!(iterator_start < codec.encode(&(partition_key.clone(), DbSortKey(vec![0, 5]))));
        assert!(iterator_start < codec.encode(&(partition_key.clone(), DbSortKey(vec![0, 5, 7]))));
        assert!(iterator_start < codec.encode(&(partition_key.clone(), DbSortKey(vec![1, 51]))));
    }

    #[test]
    fn rocksdb_partition_key_range_is_correct_for_sample_partition() {
        let codec = SubstateKeyDbCodec::default();

        let partition = DbPartitionKey {
            node_key: vec![73, 85],
            partition_num: 1,
        };
        let prev_partition = DbPartitionKey {
            node_key: vec![73, 85],
            partition_num: 0,
        };
        let prev_node = DbPartitionKey {
            node_key: vec![73, 84],
            partition_num: 255,
        };
        let next_partition = DbPartitionKey {
            node_key: vec![73, 85],
            partition_num: 2,
        };
        let next_node = DbPartitionKey {
            node_key: vec![73, 86],
            partition_num: 0,
        };

        let range = codec.encode_group_range(&partition);

        assert!(codec.encode(&(prev_partition.clone(), DbSortKey(vec![]))) < range.start);
        assert!(codec.encode(&(prev_node.clone(), DbSortKey(vec![]))) < range.start);
        assert!(codec.encode(&(partition.clone(), DbSortKey(vec![]))) >= range.start);
        assert!(codec.encode(&(partition.clone(), DbSortKey(vec![]))) < range.end);
        assert!(codec.encode(&(partition.clone(), DbSortKey(vec![1]))) >= range.start);
        assert!(codec.encode(&(partition.clone(), DbSortKey(vec![1]))) < range.end);
        assert!(codec.encode(&(next_partition.clone(), DbSortKey(vec![]))) >= range.end);
        assert!(codec.encode(&(next_node.clone(), DbSortKey(vec![]))) >= range.end);
    }

    #[test]
    fn rocksdb_partition_key_range_is_correct_for_max_partition_num_of_a_sample_node() {
        let codec = SubstateKeyDbCodec::default();

        let partition = DbPartitionKey {
            node_key: vec![73, 85],
            partition_num: 255,
        };
        let prev_partition = DbPartitionKey {
            node_key: vec![73, 85],
            partition_num: 254,
        };
        let prev_node = DbPartitionKey {
            node_key: vec![73, 84],
            partition_num: 255,
        };
        let next_partition = DbPartitionKey {
            node_key: vec![73, 86],
            partition_num: 0,
        };
        let next_node = DbPartitionKey {
            node_key: vec![73, 86],
            partition_num: 0,
        };

        let range = codec.encode_group_range(&partition);

        assert!(codec.encode(&(prev_partition.clone(), DbSortKey(vec![]))) < range.start);
        assert!(codec.encode(&(prev_node.clone(), DbSortKey(vec![]))) < range.start);
        assert!(codec.encode(&(partition.clone(), DbSortKey(vec![]))) >= range.start);
        assert!(codec.encode(&(partition.clone(), DbSortKey(vec![]))) < range.end);
        assert!(codec.encode(&(partition.clone(), DbSortKey(vec![1]))) >= range.start);
        assert!(codec.encode(&(partition.clone(), DbSortKey(vec![1]))) < range.end);
        assert!(codec.encode(&(next_partition.clone(), DbSortKey(vec![]))) >= range.end);
        assert!(codec.encode(&(next_node.clone(), DbSortKey(vec![]))) >= range.end);
    }

    #[test]
    fn rocksdb_partition_key_range_is_correct_for_max_partition_num_of_max_node() {
        let codec = SubstateKeyDbCodec::default();

        let partition = DbPartitionKey {
            node_key: vec![255, 255],
            partition_num: 255,
        };
        let prev_partition = DbPartitionKey {
            node_key: vec![255, 255],
            partition_num: 254,
        };
        let prev_node = DbPartitionKey {
            node_key: vec![255, 254],
            partition_num: 255,
        };

        let range = codec.encode_group_range(&partition);

        assert!(codec.encode(&(prev_partition.clone(), DbSortKey(vec![]))) < range.start);
        assert!(codec.encode(&(prev_node.clone(), DbSortKey(vec![]))) < range.start);
        assert!(codec.encode(&(partition.clone(), DbSortKey(vec![]))) >= range.start);
        assert!(codec.encode(&(partition.clone(), DbSortKey(vec![1]))) >= range.start);
        // we cannot test against `next_node`, but we can assert that a super-large substate key is covered:
        assert!(codec.encode(&(partition.clone(), DbSortKey(vec![255; 1000]))) < range.end);
    }
}
