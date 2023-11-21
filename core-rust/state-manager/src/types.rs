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

use crate::accumulator_tree::IsMerklizableHash;
use crate::transaction::*;
use crate::{LedgerTransactionOutcome, PartitionChange, SubstateChange};
use radix_engine::types::*;
use radix_engine_common::prelude::IsHash;
use std::fmt;
use std::fmt::Formatter;
use std::mem::size_of;
use std::num::TryFromIntError;
use std::ops::Range;
use transaction::prelude::*;

use transaction::signing::secp256k1::Secp256k1Signature;

/// A complete ID of a Substate.
#[derive(Debug, Clone, Hash, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct SubstateReference(pub NodeId, pub PartitionNumber, pub SubstateKey);

/// A complete ID of a Partition.
#[derive(Debug, Clone, Hash, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct PartitionReference(pub NodeId, pub PartitionNumber);

define_wrapped_hash!(StateChangeHash);

impl Display for StateChangeHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl StateChangeHash {
    pub fn from_substate_change(substate_change: &SubstateChange) -> StateChangeHash {
        StateChangeHash(hash(scrypto_encode(substate_change).unwrap()))
    }

    pub fn from_partition_change(partition_change: &PartitionChange) -> StateChangeHash {
        StateChangeHash(hash(scrypto_encode(partition_change).unwrap()))
    }
}

impl IsMerklizableHash for StateChangeHash {}

define_wrapped_hash!(EventHash);

impl Display for EventHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl IsMerklizableHash for EventHash {}

/// A “compressed”, merklizable derivative of a `LedgerTransactionReceipt`.
/// It is of constant size, which means that some parts are included directly (simple fields, e.g.
/// the boolean outcome) while the rest is included via merkle root hashes (collections, e.g.
/// substate changes).
/// This receipt (i.e. its SBOR serialization) is directly used for computing a `LedgerReceiptHash`.
#[derive(ScryptoCategorize, ScryptoEncode)]
pub struct ConsensusReceipt {
    /// The high-level outcome from the `LedgerTransactionReceipt`.
    pub outcome: LedgerTransactionOutcome,
    /// The root hash of a merkle tree whose leaves are hashes of the `LedgerTransactionReceipt`'s
    /// `substate_changes`.
    pub substate_change_root: StateChangeHash,
    /// The root hash of a merkle tree whose leaves are hashes of the `LedgerTransactionReceipt`'s
    /// `application_events` (see `ApplicationEvent::get_hash()`).
    pub event_root: EventHash,
}

impl ConsensusReceipt {
    pub fn get_hash(&self) -> LedgerReceiptHash {
        LedgerReceiptHash::from(blake2b_256_hash(scrypto_encode(self).unwrap()))
    }
}

define_wrapped_hash! {
    /// A hash of an SBOR-encoded `ConsensusReceipt`, capturing all the critical, on-ledger effects of
    /// executing a transaction.
    /// This is the hash that consensus agrees on.
    LedgerReceiptHash
}

impl Display for LedgerReceiptHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

define_wrapped_hash! {
    StateHash
}

impl Display for StateHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

define_wrapped_hash! {
    TransactionTreeHash
}

impl Display for TransactionTreeHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<LedgerTransactionHash> for TransactionTreeHash {
    fn from(hash: LedgerTransactionHash) -> Self {
        Self::from(hash.into_hash())
    }
}

impl IsMerklizableHash for TransactionTreeHash {}

define_wrapped_hash! {
    ReceiptTreeHash
}

impl Display for ReceiptTreeHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<LedgerReceiptHash> for ReceiptTreeHash {
    fn from(hash: LedgerReceiptHash) -> Self {
        Self::from(hash.into_hash())
    }
}

impl IsMerklizableHash for ReceiptTreeHash {}

/// A type-safe state version number.
#[derive(PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord, Debug, Sbor)]
#[sbor(transparent)]
pub struct StateVersion(u64);

/// A difference between two [`StateVersion`]s.
/// It can be negative, and it technically would fit in a (hypothetical) `i65`, so we use `i128`.
pub type StateVersionDelta = i128;

/// A forward progress from one [`StateVersion`] to another.
/// It is simply a difference (`to - from`, possibly 0); cannot be negative.
pub type StateVersionProgress = u64;

impl StateVersion {
    /// A number of bytes needed to express a version.
    pub const BYTE_LEN: usize = size_of::<u64>();

    /// A conventional version assumed before any genesis transaction.
    pub fn pre_genesis() -> Self {
        Self(0)
    }

    /// Parses the given big-endian bytes to a version.
    pub fn from_be_bytes(be_bytes: impl AsRef<[u8]>) -> Self {
        Self(u64::from_be_bytes(be_bytes.as_ref().try_into().unwrap()))
    }

    /// Converts this version to big-endian bytes.
    pub fn to_be_bytes(self) -> [u8; StateVersion::BYTE_LEN] {
        self.0.to_be_bytes()
    }

    /// Creates a version from a direct number.
    pub fn of(number: u64) -> Self {
        Self(number)
    }

    /// Returns a direct number.
    pub fn number(&self) -> u64 {
        self.0
    }

    /// Creates an immdiate predecessor.
    /// Returns error on underflow.
    pub fn previous(&self) -> Result<Self, TryFromIntError> {
        self.relative(-1)
    }

    /// Creates an immediate successor version.
    /// Returns error on overflow.
    pub fn next(&self) -> Result<Self, TryFromIntError> {
        self.relative(1)
    }

    /// Creates a version relative to this one.
    /// Returns error on overflow or underflow.
    pub fn relative(&self, delta: impl Into<StateVersionDelta>) -> Result<Self, TryFromIntError> {
        let number = self.0 as i128; // every u64 is safe to represent as i128
        let delta_number = delta.into();
        let relative_number = number
            .checked_add(delta_number)
            .expect("both operands are representable by i65, so their sum must fit in i128");
        match u64::try_from(relative_number) {
            Ok(relative_number) => Ok(Self(relative_number)),
            Err(error) => Err(error),
        }
    }

    /// Creates an iterator of all versions starting with this one, and ending at the given one
    /// (exclusive).
    /// This is an equivalent of a (hypothetical) `self..end` syntax (which is forbidden to
    /// implement due to crate restrictions).
    pub fn to(&self, end: StateVersion) -> impl Iterator<Item = StateVersion> {
        (self.0..end.0).map(StateVersion::of)
    }

    /// Returns a number of state versions between `from` and `to`, or `Err` if the progress would
    /// be negative.
    pub fn calculate_progress(
        from: StateVersion,
        to: StateVersion,
    ) -> Result<StateVersionProgress, TryFromIntError> {
        u64::try_from((to.0 as i128) - (from.0 as i128))
    }
}

impl Display for StateVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord, Debug, Sbor)]
pub struct LedgerHashes {
    pub state_root: StateHash,
    pub transaction_root: TransactionTreeHash,
    pub receipt_root: ReceiptTreeHash,
}

impl LedgerHashes {
    pub fn pre_genesis() -> Self {
        Self {
            state_root: StateHash(Hash([0; Hash::LENGTH])),
            transaction_root: TransactionTreeHash(Hash([0; Hash::LENGTH])),
            receipt_root: ReceiptTreeHash(Hash([0; Hash::LENGTH])),
        }
    }
}

#[derive(Debug)]
pub struct PreviewRequest {
    pub manifest: TransactionManifestV1,
    pub explicit_epoch_range: Option<Range<Epoch>>,
    pub notary_public_key: Option<PublicKey>,
    pub notary_is_signatory: bool,
    pub tip_percentage: u16,
    pub nonce: u32,
    pub signer_public_keys: Vec<PublicKey>,
    pub flags: PreviewFlags,
    pub message: MessageV1,
}

#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum InvalidCommitRequestError {
    TransactionParsingFailed,
    TransactionRootMismatch,
}

#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct CommitRequest {
    pub transactions: Vec<RawLedgerTransaction>,
    pub proof: LedgerProof,
    pub vertex_store: Option<Vec<u8>>,
    pub self_validator_id: Option<ValidatorId>, // for metrics calculation only
}

#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct CommitSummary {
    pub validator_round_counters: Vec<(ValidatorId, LeaderRoundCounter)>,
    pub num_user_transactions: u32,
}

#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct PrepareRequest {
    pub committed_ledger_hashes: LedgerHashes,
    pub ancestor_transactions: Vec<RawLedgerTransaction>,
    pub ancestor_ledger_hashes: LedgerHashes,
    pub proposed_transactions: Vec<RawNotarizedTransaction>,
    pub round_history: RoundHistory,
}

#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct RoundHistory {
    pub is_fallback: bool,
    pub epoch: Epoch,
    pub round: Round,
    pub gap_round_leader_addresses: Vec<ComponentAddress>,
    pub proposer_address: ComponentAddress,
    pub proposer_timestamp_ms: i64,
}

#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct PrepareResult {
    pub committed: Vec<CommittableTransaction>,
    /// Note: this is only used for testing
    pub rejected: Vec<RejectedTransaction>,
    pub next_epoch: Option<NextEpoch>,
    pub next_protocol_version: Option<String>,
    pub ledger_hashes: LedgerHashes,
}

#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct CommittableTransaction {
    /// Not included for the Round Change transaction which is inserted and doesn't come from the proposal
    pub index: Option<u32>,
    pub raw: RawLedgerTransaction,
    pub intent_hash: Option<IntentHash>,
    pub notarized_transaction_hash: Option<NotarizedTransactionHash>,
    pub ledger_transaction_hash: LedgerTransactionHash,
}

#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct RejectedTransaction {
    pub index: u32,
    // Note - these are None if the transaction can't even be prepared to determine the hashes
    pub intent_hash: Option<IntentHash>,
    pub notarized_transaction_hash: Option<NotarizedTransactionHash>,
    pub ledger_transaction_hash: Option<LedgerTransactionHash>,
    pub error: String,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ActiveValidatorInfo {
    pub address: ComponentAddress,
    pub key: Secp256k1PublicKey,
    pub stake: Decimal,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct NextEpoch {
    pub epoch: Epoch,
    pub validator_set: Vec<ActiveValidatorInfo>,
}

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct TimestampedValidatorSignature {
    pub key: Secp256k1PublicKey,
    pub validator_address: ComponentAddress,
    pub timestamp_ms: i64,
    pub signature: Secp256k1Signature,
}

define_single_versioned! {
    #[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
    pub enum VersionedLedgerProof => LedgerProof = LedgerProofV1
}

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct LedgerProofV1 {
    pub opaque: Hash,
    pub ledger_header: LedgerHeader,
    pub timestamped_signatures: Vec<TimestampedValidatorSignature>,
}

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct LedgerHeader {
    pub epoch: Epoch,
    pub round: Round,
    pub state_version: StateVersion,
    pub hashes: LedgerHashes,
    pub consensus_parent_round_timestamp_ms: i64,
    pub proposer_timestamp_ms: i64,
    pub next_epoch: Option<NextEpoch>,
    pub next_protocol_version: Option<String>,
}

pub struct EpochTransactionIdentifiers {
    pub state_version: StateVersion,
    pub transaction_hash: TransactionTreeHash,
    pub receipt_hash: ReceiptTreeHash,
}

impl EpochTransactionIdentifiers {
    pub fn pre_genesis() -> Self {
        let ledger_hashes = LedgerHashes::pre_genesis();
        Self {
            state_version: StateVersion::pre_genesis(),
            transaction_hash: ledger_hashes.transaction_root,
            receipt_hash: ledger_hashes.receipt_root,
        }
    }

    pub fn from(epoch_header: &LedgerHeader) -> Self {
        Self {
            state_version: epoch_header.state_version,
            transaction_hash: epoch_header.hashes.transaction_root,
            receipt_hash: epoch_header.hashes.receipt_root,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ValidatorId {
    pub component_address: ComponentAddress,
    pub key: Secp256k1PublicKey,
}

impl ValidatorId {
    pub fn from(active_validator_info: &ActiveValidatorInfo) -> ValidatorId {
        ValidatorId {
            component_address: active_validator_info.address,
            key: active_validator_info.key,
        }
    }
}
