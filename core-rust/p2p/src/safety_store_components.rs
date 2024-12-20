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

use crate::prelude::*;
use sbor::define_single_versioned;

#[derive(Debug, Clone, ScryptoSbor)]
pub struct SafetyStateV1 {
    validator_id: BFTValidatorId,
    round: Round,
    last_vote: Option<Vote>,
}

define_single_versioned! {
    #[derive(Debug, Clone, ScryptoSbor)]
    pub VersionedSafetyState(SafetyStateVersions) => SafetyState = SafetyStateV1,
    outer_attributes: [
        #[derive(ScryptoSborAssertion)]
        #[sbor_assert(backwards_compatible(
            cuttlefish = "FILE:CF_SCHEMA_versioned_safety_state_cuttlefish.bin"
        ))]
    ]
}

/// Safety state components. Note that these structs are intended only for proper encoding/deconding
/// of the safety state. They may repeat existing structs defined elsewhere.

/// Timestamp of the various safety state components.
// At present it's just an alias for i64. Later we may want to replace it with struct using crono crate and
// do something like shown below to transparently convert to/from internal representation
// (once there will be real usage at Rust side).
// #[sbor(
//     as_type = "i64",
//     as_ref = "self.timestamp()",
//     from_value = "Self(DateTime::from_timestamp(value, 0))"
// )]
type SafetyStateTimestamp = i64;

#[derive(Debug, Clone, ScryptoSbor)]
pub struct BFTHeader {
    round: Round,
    vertex_id: VertexId,
    ledger_header: LedgerHeader,
}

#[derive(Debug, Clone, ScryptoSbor, Ord, PartialOrd, Eq, PartialEq)]
pub struct BFTValidator {
    power: Vec<u8>,
    validator_id: BFTValidatorId,
}

#[derive(Debug, Clone, ScryptoSbor, Ord, PartialOrd, Eq, PartialEq)]
pub struct BFTValidatorId {
    key: NodeSecp256k1PublicKey,
    validator_address: ComponentAddress,
}

#[derive(Debug, Clone, ScryptoSbor)]
pub struct HighQC {
    highest_quorum_certificate: QuorumCertificate,
    highest_committed_quorum_certificate: QuorumCertificate,
    highest_timeout_certificate: Option<TimeoutCertificate>,
}

// FIXME: A duplicate of LedgerHeader from StateManager.
// Made separate te reference only types within this module. De-duplication requires
// careful merging of the other referenced types as well.
#[derive(Debug, Clone, ScryptoSbor)]
pub struct LedgerHeader {
    epoch: i64,
    round: Round,
    state_version: i64,
    hashes: LedgerHashes,
    consensus_parent_round_timestamp_ms: SafetyStateTimestamp,
    proposer_timestamp_ms: SafetyStateTimestamp,
    next_epoch: Option<NextEpoch>,
    next_protocol_version: Option<String>,
}

#[derive(Debug, Clone, ScryptoSbor)]
pub struct LedgerHashes {
    pub state_root: RawHash,
    pub transaction_root: RawHash,
    pub receipt_root: RawHash,
}

define_wrapped_hash! {
    RawHash
}

#[derive(Debug, Clone, ScryptoSbor)]
pub struct NextEpoch {
    epoch: i64,
    validators: Vec<BFTValidator>,
}

#[derive(Debug, Clone, ScryptoSbor)]
pub struct QuorumCertificate {
    signatures: TimestampedECDSASignatures,
    vote_data: VoteData,
}

#[derive(Debug, Clone, ScryptoSbor)]
pub struct Round {
    round: i64,
}

#[derive(Debug, Clone, ScryptoSbor)]
pub struct TimeoutCertificate {
    epoch: i64,
    round: Round,
    signatures: TimestampedECDSASignatures,
}

#[derive(Debug, Clone, ScryptoSbor)]
pub struct TimestampedECDSASignature {
    timestamp: SafetyStateTimestamp,
    signature: NodeSignature,
}

#[derive(Debug, Clone, ScryptoSbor)]
pub struct TimestampedECDSASignatures {
    node_to_timestamped_signature: BTreeMap<BFTValidatorId, TimestampedECDSASignature>,
}

#[derive(Debug, Clone, ScryptoSbor)]
pub struct VertexId {
    id_bytes: Vec<u8>,
}

#[derive(Debug, Clone, ScryptoSbor)]
pub struct Vote {
    author: BFTValidatorId,
    high_quorum_certificate: HighQC,
    vote_data: VoteData,
    timestamp: SafetyStateTimestamp,
    signature: NodeSignature,
    timeout_signature: Option<NodeSignature>,
}

#[derive(Debug, Clone, ScryptoSbor)]
pub struct VoteData {
    proposed: BFTHeader,
    parent: BFTHeader,
    committed: Option<BFTHeader>,
}
