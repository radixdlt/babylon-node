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

use std::collections::HashSet;

use crate::jni::state_manager::ActualStateManager;

use jni::objects::{JClass, JObject};
use jni::sys::jbyteArray;
use jni::JNIEnv;
use radix_engine::types::scrypto_encode;
use sbor::{Categorize, Decode, Encode};
use transaction::errors::TransactionValidationError;

use crate::jni::utils::*;
use crate::transaction::UserTransactionValidator;
use crate::types::PendingTransaction;
use crate::{mempool::*, LedgerPayloadHash, UserPayloadHash};

//
// JNI Interface
//

#[no_mangle]
extern "system" fn Java_com_radixdlt_mempool_RustMempool_add(
    env: JNIEnv,
    _class: JClass,
    j_state_manager: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_state_manager_sbor_call(env, j_state_manager, request_payload, do_add)
}

#[tracing::instrument(skip_all)]
fn do_add(
    state_manager: &mut ActualStateManager,
    transaction: JavaRawTransaction,
) -> Result<JavaPayloadHash, MempoolAddErrorJava> {
    let notarized_transaction =
        UserTransactionValidator::parse_unvalidated_user_transaction_from_slice(
            &transaction.payload,
        )?;

    state_manager
        .check_for_rejection_and_add_to_mempool(
            MempoolAddSource::MempoolSync,
            notarized_transaction,
        )
        .map(|_| transaction.payload_hash)
        .map_err(Into::into)
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_mempool_RustMempool_getTransactionsForProposal(
    env: JNIEnv,
    _class: JClass,
    j_state_manager: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_state_manager_sbor_read_call(
        env,
        j_state_manager,
        request_payload,
        do_get_transactions_for_proposal,
    )
}

#[tracing::instrument(skip_all)]
fn do_get_transactions_for_proposal(
    state_manager: &ActualStateManager,
    (count, prepared_transactions): (u32, Vec<JavaPayloadHash>),
) -> Vec<JavaRawTransaction> {
    let prepared_ids: HashSet<UserPayloadHash> = prepared_transactions
        .into_iter()
        .map(|id| {
            UserPayloadHash::from_raw_bytes(
                id.0.try_into().expect("transaction id the wrong length"),
            )
        })
        .collect();

    state_manager
        .mempool
        .get_proposal_transactions(count.into(), &prepared_ids)
        .into_iter()
        .map(|t| t.into())
        .collect()
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_mempool_RustMempool_getCount(
    env: JNIEnv,
    _class: JClass,
    j_state_manager: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_state_manager_sbor_read_call(env, j_state_manager, request_payload, do_get_count)
}

#[tracing::instrument(skip_all)]
fn do_get_count(state_manager: &ActualStateManager, _args: ()) -> i32 {
    state_manager.mempool.get_count().try_into().unwrap()
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_mempool_RustMempool_getTransactionsToRelay(
    env: JNIEnv,
    _class: JClass,
    j_state_manager: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_state_manager_sbor_call(
        env,
        j_state_manager,
        request_payload,
        do_get_transactions_to_relay,
    )
}

#[tracing::instrument(skip_all)]
fn do_get_transactions_to_relay(
    state_manager: &mut ActualStateManager,
    _args: (),
) -> Vec<JavaRawTransaction> {
    state_manager
        .get_relay_transactions()
        .into_iter()
        .map(|t| t.into())
        .collect()
}

//
// DTO Models + Mapping
//

/// Corresponds to the payload_hash
#[derive(Debug, PartialEq, Eq, Categorize, Encode, Decode)]
pub struct JavaPayloadHash(Vec<u8>);

impl From<LedgerPayloadHash> for JavaPayloadHash {
    fn from(payload_hash: LedgerPayloadHash) -> Self {
        JavaPayloadHash(payload_hash.into_bytes().to_vec())
    }
}

#[derive(Debug, Categorize, Encode, Decode)]
pub struct JavaRawTransaction {
    pub payload: Vec<u8>,
    pub payload_hash: JavaPayloadHash,
}

impl From<PendingTransaction> for JavaRawTransaction {
    fn from(transaction: PendingTransaction) -> Self {
        JavaRawTransaction {
            payload: scrypto_encode(&transaction.payload).unwrap(),
            payload_hash: JavaPayloadHash(transaction.payload_hash.into_bytes().to_vec()),
        }
    }
}

#[derive(Debug, Categorize, Encode, Decode)]
enum MempoolAddErrorJava {
    Full { current_size: u64, max_size: u64 },
    Duplicate,
    TransactionValidationError(String),
    Rejected(String),
}

impl From<MempoolAddError> for MempoolAddErrorJava {
    fn from(err: MempoolAddError) -> Self {
        match err {
            MempoolAddError::Full {
                current_size,
                max_size,
            } => MempoolAddErrorJava::Full {
                current_size,
                max_size,
            },
            MempoolAddError::Duplicate => MempoolAddErrorJava::Duplicate,
            MempoolAddError::Rejected(rejection) => {
                MempoolAddErrorJava::Rejected(rejection.reason.to_string())
            }
        }
    }
}

impl From<TransactionValidationError> for MempoolAddErrorJava {
    fn from(error: TransactionValidationError) -> Self {
        MempoolAddErrorJava::TransactionValidationError(format!("{:?}", error))
    }
}

pub fn export_extern_functions() {}
