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

use scrypto::prelude::*;
use std::fmt;
use transaction::model::{
    NotarizedTransaction, PreviewFlags, TransactionIntent, TransactionManifest,
};

#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord, Decode, Encode, TypeId)]
pub struct PayloadHash(pub [u8; Self::LENGTH]);

impl PayloadHash {
    pub const LENGTH: usize = 32;
}

impl fmt::Display for PayloadHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}

impl From<Hash> for PayloadHash {
    fn from(hash: Hash) -> Self {
        PayloadHash(hash.0)
    }
}

impl From<&NotarizedTransaction> for PayloadHash {
    fn from(transaction: &NotarizedTransaction) -> Self {
        PayloadHash(sha256_twice(scrypto_encode(transaction)).0)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord, Decode, Encode, TypeId)]
pub struct IntentHash(pub [u8; Self::LENGTH]);

impl IntentHash {
    pub const LENGTH: usize = 32;
}

impl fmt::Display for IntentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}

impl From<Hash> for IntentHash {
    fn from(hash: Hash) -> Self {
        IntentHash(hash.0)
    }
}

impl From<&TransactionIntent> for IntentHash {
    fn from(intent: &TransactionIntent) -> Self {
        IntentHash(sha256_twice(&scrypto_encode(intent)).0)
    }
}

impl From<&NotarizedTransaction> for IntentHash {
    fn from(transaction: &NotarizedTransaction) -> Self {
        let intent = &transaction.signed_intent.intent;
        intent.into()
    }
}

/// An uncommitted user transaction, in eg the mempool
#[derive(Debug, PartialEq, Eq, Clone, Decode, Encode, TypeId)]
pub struct PendingTransaction {
    pub payload: NotarizedTransaction,
    pub payload_hash: PayloadHash,
    pub intent_hash: IntentHash,
}

impl From<NotarizedTransaction> for PendingTransaction {
    fn from(transaction: NotarizedTransaction) -> Self {
        let payload_hash: PayloadHash = (&transaction).into();
        let intent_hash: IntentHash = (&transaction).into();
        PendingTransaction {
            payload: transaction,
            payload_hash,
            intent_hash,
        }
    }
}

/// A transaction for persisting - eg in the Database or in a block
#[derive(Debug, PartialEq, Eq, Clone, Decode, Encode, TypeId)]
pub enum StoredTransaction {
    User(NotarizedTransaction),
    System(Vec<u8>), // Just a payload for now. Todo - something better soon?
}

impl StoredTransaction {
    pub fn get_hash(&self) -> PayloadHash {
        match self {
            StoredTransaction::User(notarized) => notarized.into(),
            StoredTransaction::System(payload) => PayloadHash(sha256_twice(&payload).0),
        }
    }

    pub fn into_payload(self) -> Vec<u8> {
        match self {
            StoredTransaction::User(notarized) => scrypto_encode(&notarized),
            StoredTransaction::System(payload) => payload,
        }
    }
}

#[derive(Debug, Decode, Encode, TypeId)]
pub struct PreviewRequest {
    pub manifest: TransactionManifest,
    pub cost_unit_limit: u32,
    pub tip_percentage: u32,
    pub nonce: u64,
    pub signer_public_keys: Vec<PublicKey>,
    pub flags: PreviewFlags,
}

#[derive(Debug, Decode, Encode, TypeId)]
pub struct CommitRequest {
    pub transaction_payloads: Vec<Vec<u8>>,
    pub state_version: u64,
    pub proof: Vec<u8>,
    pub vertex_store: Option<Vec<u8>>,
}

#[derive(Debug, Decode, Encode, TypeId)]
pub struct PrepareRequest {
    pub already_prepared_payloads: Vec<Vec<u8>>,
    pub proposed_payloads: Vec<Vec<u8>>,
}

#[derive(Debug, Decode, Encode, TypeId)]
pub enum TransactionPrepareResult {
    CanCommit,
    Reject { reason: String },
}

#[derive(Debug, Decode, Encode, TypeId)]
pub struct PrepareResult {
    pub transaction_results: Vec<(PayloadHash, TransactionPrepareResult)>,
}
