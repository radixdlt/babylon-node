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

use crate::transaction::LedgerTransaction;
use radix_engine::model::Validator;
use radix_engine::types::{
    scrypto, scrypto_encode, sha256_twice, Decode, Encode, Hash, PublicKey, TypeId,
};
use std::collections::BTreeSet;
use std::fmt;
use transaction::model::{
    NotarizedTransaction, PreviewFlags, SignedTransactionIntent, TransactionIntent,
    TransactionManifest,
};

#[derive(PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord, Decode, Encode, TypeId)]
pub struct AccumulatorHash([u8; Self::LENGTH]);

impl AccumulatorHash {
    pub const LENGTH: usize = 32;

    pub fn pre_genesis() -> Self {
        Self([0; Self::LENGTH])
    }

    pub fn accumulate(&self, ledger_payload_hash: &LedgerPayloadHash) -> Self {
        let concat_bytes = [self.0, ledger_payload_hash.0].concat();
        Self(sha256_twice(concat_bytes).0)
    }

    pub fn from_raw_bytes(hash_bytes: [u8; Self::LENGTH]) -> Self {
        Self(hash_bytes)
    }

    pub fn into_bytes(self) -> [u8; Self::LENGTH] {
        self.0
    }
}

impl AsRef<[u8]> for AccumulatorHash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Display for AccumulatorHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl fmt::Debug for AccumulatorHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("AccumulatorHash")
            .field(&hex::encode(self.0))
            .finish()
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
#[scrypto(TypeId, Encode, Decode)]
pub struct LedgerPayloadHash([u8; Self::LENGTH]);

impl LedgerPayloadHash {
    pub const LENGTH: usize = 32;

    pub fn for_transaction(transaction: &LedgerTransaction) -> Self {
        Self(sha256_twice(scrypto_encode(transaction).unwrap()).0)
    }

    pub fn from_raw_bytes(hash_bytes: [u8; Self::LENGTH]) -> Self {
        Self(hash_bytes)
    }

    pub fn into_bytes(self) -> [u8; Self::LENGTH] {
        self.0
    }
}

impl AsRef<[u8]> for LedgerPayloadHash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<Hash> for LedgerPayloadHash {
    fn from(hash: Hash) -> Self {
        LedgerPayloadHash(hash.0)
    }
}

impl fmt::Display for LedgerPayloadHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl fmt::Debug for LedgerPayloadHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("LedgerPayloadHash")
            .field(&hex::encode(self.0))
            .finish()
    }
}

pub trait HasLedgerPayloadHash {
    fn ledger_payload_hash(&self) -> LedgerPayloadHash;
}

impl HasLedgerPayloadHash for LedgerTransaction {
    fn ledger_payload_hash(&self) -> LedgerPayloadHash {
        LedgerPayloadHash::for_transaction(self)
    }
}

impl HasLedgerPayloadHash for NotarizedTransaction {
    fn ledger_payload_hash(&self) -> LedgerPayloadHash {
        // Could optimize this to remove the clone in future,
        // once SBOR/models are more stable
        LedgerTransaction::User(self.clone()).ledger_payload_hash()
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
#[scrypto(TypeId, Encode, Decode)]
pub struct UserPayloadHash([u8; Self::LENGTH]);

impl UserPayloadHash {
    pub const LENGTH: usize = 32;

    pub fn for_transaction(transaction: &NotarizedTransaction) -> Self {
        Self(sha256_twice(scrypto_encode(transaction).unwrap()).0)
    }

    pub fn from_raw_bytes(hash_bytes: [u8; Self::LENGTH]) -> Self {
        Self(hash_bytes)
    }

    pub fn into_bytes(self) -> [u8; Self::LENGTH] {
        self.0
    }
}

impl AsRef<[u8]> for UserPayloadHash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Display for UserPayloadHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl fmt::Debug for UserPayloadHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("UserPayloadHash")
            .field(&hex::encode(self.0))
            .finish()
    }
}

pub trait HasUserPayloadHash {
    fn user_payload_hash(&self) -> UserPayloadHash;
}

impl HasUserPayloadHash for NotarizedTransaction {
    fn user_payload_hash(&self) -> UserPayloadHash {
        UserPayloadHash::for_transaction(self)
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
#[scrypto(TypeId, Encode, Decode)]
pub struct SignaturesHash([u8; Self::LENGTH]);

impl SignaturesHash {
    pub const LENGTH: usize = 32;

    pub fn for_signed_intent(signed_intent: &SignedTransactionIntent) -> Self {
        Self(sha256_twice(scrypto_encode(signed_intent).unwrap()).0)
    }

    pub fn from_raw_bytes(hash_bytes: [u8; Self::LENGTH]) -> Self {
        Self(hash_bytes)
    }

    pub fn into_bytes(self) -> [u8; Self::LENGTH] {
        self.0
    }
}

impl AsRef<[u8]> for SignaturesHash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Display for SignaturesHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl fmt::Debug for SignaturesHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("SignaturesHash")
            .field(&hex::encode(self.0))
            .finish()
    }
}

pub trait HasSignaturesHash {
    fn signatures_hash(&self) -> SignaturesHash;
}

impl HasSignaturesHash for SignedTransactionIntent {
    fn signatures_hash(&self) -> SignaturesHash {
        SignaturesHash::for_signed_intent(self)
    }
}

impl HasSignaturesHash for NotarizedTransaction {
    fn signatures_hash(&self) -> SignaturesHash {
        self.signed_intent.signatures_hash()
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
#[scrypto(TypeId, Encode, Decode)]
pub struct IntentHash([u8; Self::LENGTH]);

impl IntentHash {
    pub const LENGTH: usize = 32;

    pub fn for_intent(intent: &TransactionIntent) -> Self {
        Self(sha256_twice(scrypto_encode(intent).unwrap()).0)
    }

    pub fn from_raw_bytes(hash_bytes: [u8; Self::LENGTH]) -> Self {
        Self(hash_bytes)
    }

    pub fn into_bytes(self) -> [u8; Self::LENGTH] {
        self.0
    }
}

impl AsRef<[u8]> for IntentHash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Display for IntentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl fmt::Debug for IntentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("IntentHash")
            .field(&hex::encode(self.0))
            .finish()
    }
}

pub trait HasIntentHash {
    fn intent_hash(&self) -> IntentHash;
}

impl HasIntentHash for TransactionIntent {
    fn intent_hash(&self) -> IntentHash {
        IntentHash::for_intent(self)
    }
}

impl HasIntentHash for SignedTransactionIntent {
    fn intent_hash(&self) -> IntentHash {
        self.intent.intent_hash()
    }
}

impl HasIntentHash for NotarizedTransaction {
    fn intent_hash(&self) -> IntentHash {
        self.signed_intent.intent.intent_hash()
    }
}

/// An uncommitted user transaction, in eg the mempool
#[derive(Debug, PartialEq, Eq, Clone)]
#[scrypto(TypeId, Encode, Decode)]
pub struct PendingTransaction {
    pub payload: NotarizedTransaction,
    pub payload_hash: UserPayloadHash,
    pub intent_hash: IntentHash,
}

impl From<NotarizedTransaction> for PendingTransaction {
    fn from(transaction: NotarizedTransaction) -> Self {
        let intent_hash = transaction.intent_hash();
        PendingTransaction {
            payload_hash: transaction.user_payload_hash(),
            intent_hash,
            payload: transaction,
        }
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct PreviewRequest {
    pub manifest: TransactionManifest,
    pub start_epoch_inclusive: u64,
    pub end_epoch_exclusive: u64,
    pub notary_public_key: Option<PublicKey>,
    pub notary_as_signatory: bool,
    pub cost_unit_limit: u32,
    pub tip_percentage: u16,
    pub nonce: u64,
    pub signer_public_keys: Vec<PublicKey>,
    pub flags: PreviewFlags,
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub enum CommitError {
    MissingEpochProof,
}

#[derive(Debug, Decode, Encode, TypeId)]
pub struct CommitRequest {
    pub transaction_payloads: Vec<Vec<u8>>,
    pub proof_state_version: u64, // TODO: Use actual proof to get this info
    pub proof: Vec<u8>,
    pub vertex_store: Option<Vec<u8>>,
}

#[derive(Debug, Decode, Encode, TypeId)]
pub struct PrepareRequest {
    pub already_prepared_payloads: Vec<Vec<u8>>,
    pub proposed_payloads: Vec<Vec<u8>>,
    pub consensus_epoch: u64,
    pub round_number: u64,
    pub proposer_timestamp_ms: i64,
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct PrepareResult {
    pub committed: Vec<Vec<u8>>,
    pub rejected: Vec<(Vec<u8>, String)>,
    pub next_epoch: Option<NextEpoch>,
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct NextEpoch {
    pub validator_set: BTreeSet<Validator>,
    pub epoch: u64,
}

#[derive(Debug, Decode, Encode, TypeId)]
pub struct PrepareGenesisRequest {
    pub genesis: Vec<u8>,
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct PrepareGenesisResult {
    pub validator_set: Option<BTreeSet<Validator>>,
}
