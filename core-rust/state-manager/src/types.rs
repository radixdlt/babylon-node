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
use crate::{CommitBasedIdentifiers, LedgerTransactionOutcome, SubstateChange};
use radix_engine::types::*;
use radix_engine_common::prelude::IsHash;
use std::fmt;
use std::ops::Range;
use transaction::prelude::*;

use transaction::ecdsa_secp256k1::EcdsaSecp256k1Signature;

define_wrapped_hash!(AccumulatorHash);

impl AccumulatorHash {
    pub fn pre_genesis() -> Self {
        Self(Hash([0; Hash::LENGTH]))
    }

    pub fn accumulate(&self, ledger_payload_hash: &LegacyLedgerPayloadHash) -> Self {
        let concat_bytes = [self.0.as_slice(), ledger_payload_hash.as_slice()].concat();
        Self(blake2b_256_hash(concat_bytes))
    }
}

define_wrapped_hash!(SubstateChangeHash);

impl SubstateChangeHash {
    pub fn from_substate_change(substate_change: &SubstateChange) -> SubstateChangeHash {
        SubstateChangeHash(hash(scrypto_encode(&substate_change).unwrap()))
    }
}

impl IsMerklizableHash for SubstateChangeHash {}

define_wrapped_hash!(EventHash);

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
    pub substate_change_root: SubstateChangeHash,
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

define_wrapped_hash! {
    StateHash
}

define_wrapped_hash! {
    TransactionTreeHash
}

impl IsMerklizableHash for TransactionTreeHash {}

define_wrapped_hash! {
    ReceiptTreeHash
}

impl IsMerklizableHash for ReceiptTreeHash {}

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
    pub explicit_epoch_range: Option<Range<u64>>,
    pub notary_public_key: Option<PublicKey>,
    pub notary_is_signatory: bool,
    pub tip_percentage: u16,
    pub nonce: u32,
    pub signer_public_keys: Vec<PublicKey>,
    pub flags: PreviewFlags,
}

#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum CommitError {
    MissingEpochProof,
    SuperfluousEpochProof,
    EpochProofMismatch,
    LedgerHashesMismatch,
}

#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct CommitRequest {
    pub transaction_payloads: Vec<RawLedgerTransaction>,
    pub proof: LedgerProof,
    pub vertex_store: Option<Vec<u8>>,
}

#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct PrepareRequest {
    pub committed_accumulator_state: AccumulatorState,
    pub prepared_uncommitted_transactions: Vec<RawLedgerTransaction>,
    pub prepared_uncommitted_accumulator_state: AccumulatorState,
    pub proposed_transactions: Vec<RawNotarizedTransaction>,
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
    pub rejected: Vec<RejectedTransaction>,
    pub next_epoch: Option<NextEpoch>,
    pub ledger_hashes: LedgerHashes,
    pub accumulator_state: AccumulatorState,
}

#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct CommittableTransaction {
    /// Not included for the Round Change transaction which is inserted and doesn't come from the proposal
    pub index: Option<u32>,
    pub raw: RawLedgerTransaction,
    pub intent_hash: Option<IntentHash>,
    pub notarized_transaction_hash: Option<NotarizedTransactionHash>,
    pub ledger_hash: LedgerTransactionHash,
    pub legacy_hash: LegacyLedgerPayloadHash,
}

#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct RejectedTransaction {
    pub index: u32,
    // Note - these are None if the transaction can't even be prepared to determine the hashes
    pub intent_hash: Option<IntentHash>,
    pub notarized_transaction_hash: Option<NotarizedTransactionHash>,
    pub ledger_hash: Option<LedgerTransactionHash>,
    pub error: String,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ActiveValidatorInfo {
    pub address: ComponentAddress,
    pub key: EcdsaSecp256k1PublicKey,
    pub stake: Decimal,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct NextEpoch {
    pub epoch: Epoch,
    pub validator_set: Vec<ActiveValidatorInfo>,
}

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct TimestampedValidatorSignature {
    pub key: EcdsaSecp256k1PublicKey,
    pub validator_address: Option<ComponentAddress>,
    pub timestamp_ms: i64,
    pub signature: EcdsaSecp256k1Signature,
}

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct LedgerProof {
    pub opaque: Hash,
    pub ledger_header: LedgerHeader,
    pub timestamped_signatures: Vec<TimestampedValidatorSignature>,
}

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct LedgerHeader {
    pub epoch: Epoch,
    pub round: Round,
    pub accumulator_state: AccumulatorState,
    pub hashes: LedgerHashes,
    pub consensus_parent_round_timestamp_ms: i64,
    pub proposer_timestamp_ms: i64,
    pub next_epoch: Option<NextEpoch>,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccumulatorState {
    pub state_version: u64,
    pub accumulator_hash: AccumulatorHash,
}

impl AccumulatorState {
    pub fn new(identifiers: &CommitBasedIdentifiers) -> Self {
        Self {
            state_version: identifiers.state_version,
            accumulator_hash: identifiers.accumulator_hash,
        }
    }
}

pub struct EpochTransactionIdentifiers {
    pub state_version: u64,
    pub transaction_hash: TransactionTreeHash,
    pub receipt_hash: ReceiptTreeHash,
}

impl EpochTransactionIdentifiers {
    pub fn pre_genesis() -> Self {
        let ledger_hashes = LedgerHashes::pre_genesis();
        Self {
            state_version: 0,
            transaction_hash: ledger_hashes.transaction_root,
            receipt_hash: ledger_hashes.receipt_root,
        }
    }

    pub fn from(epoch_header: &LedgerHeader) -> Self {
        Self {
            state_version: epoch_header.accumulator_state.state_version,
            transaction_hash: epoch_header.hashes.transaction_root,
            receipt_hash: epoch_header.hashes.receipt_root,
        }
    }
}
