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

use crate::jni::state_manager::ActualStateManager;
use jni::objects::{JClass, JObject};
use jni::sys::jbyteArray;
use jni::JNIEnv;
use sbor::{Decode, Encode, TypeId};

use crate::jni::utils::*;
use crate::mempool::*;
use crate::result::{
    ResultStateManagerMaps, StateManagerError, StateManagerResult, ERRCODE_INTERFACE_CASTS,
};
use crate::types::Transaction;

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
    jni_state_manager_sbor_call_flatten_result(env, j_state_manager, request_payload, do_add)
}

fn do_add(
    state_manager: &mut ActualStateManager,
    args: Transaction,
) -> StateManagerResult<Result<Transaction, MempoolErrorJava>> {
    let transaction = args;

    // TODO: Move decoding of transaction to a separate "zone"
    // TODO: Use notarized transaction in mempool
    let decode_result = state_manager.decode_transaction(&transaction);

    if let Err(error) = decode_result {
        return Err(MempoolError::TransactionValidationError(error)).map_err_sm(|err| err.into());
    }

    state_manager
        .mempool
        .add_transaction(transaction)
        .map_err_sm(|err| err.into())
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_mempool_RustMempool_getTransactionsForProposal(
    env: JNIEnv,
    _class: JClass,
    j_state_manager: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_state_manager_sbor_call_flatten_result(
        env,
        j_state_manager,
        request_payload,
        do_get_transactions_for_proposal,
    )
}

fn do_get_transactions_for_proposal(
    state_manager: &mut ActualStateManager,
    args: (u32, Vec<Transaction>),
) -> StateManagerResult<Result<Vec<Transaction>, MempoolErrorJava>> {
    let (count, prepared_transactions) = args;
    state_manager
        .mempool
        .get_proposal_transactions(count.into(), &prepared_transactions)
        .map_err_sm(|err| err.into())
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_mempool_RustMempool_getCount(
    env: JNIEnv,
    _class: JClass,
    j_state_manager: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_state_manager_sbor_call(env, j_state_manager, request_payload, do_get_count)
}

fn do_get_count(state_manager: &mut ActualStateManager, _args: ()) -> i32 {
    state_manager.mempool.get_count().try_into().unwrap()
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_mempool_RustMempool_getTransactionsToRelay(
    env: JNIEnv,
    _class: JClass,
    j_state_manager: JObject,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_state_manager_sbor_call_flatten_result(
        env,
        j_state_manager,
        request_payload,
        do_get_transactions_to_relay,
    )
}

fn do_get_transactions_to_relay(
    state_manager: &mut ActualStateManager,
    args: (u64, u64),
) -> StateManagerResult<Result<Vec<Transaction>, MempoolErrorJava>> {
    let (initial_delay_millis, repeat_delay_millis) = args;

    state_manager
        .mempool
        .get_relay_transactions(initial_delay_millis, repeat_delay_millis)
        .map_err_sm(|err| err.into())
}

//
// DTO Models + Mapping
//

#[derive(Debug, PartialEq, TypeId, Encode, Decode)]
enum MempoolErrorJava {
    Full { current_size: i64, max_size: i64 },
    Duplicate,
    TransactionValidationError(String),
}

impl From<MempoolError> for StateManagerResult<MempoolErrorJava> {
    fn from(err: MempoolError) -> Self {
        match err {
            MempoolError::Full {
                current_size,
                max_size,
            } => Ok(MempoolErrorJava::Full {
                current_size: current_size.try_into().or_else(|_| {
                    StateManagerError::create_result(
                        ERRCODE_INTERFACE_CASTS,
                        "Failed to cast current_size".to_string(),
                    )
                })?,
                max_size: max_size.try_into().or_else(|_| {
                    StateManagerError::create_result(
                        ERRCODE_INTERFACE_CASTS,
                        "Failed to cast max_size".to_string(),
                    )
                })?,
            }),
            MempoolError::Duplicate => Ok(MempoolErrorJava::Duplicate),
            MempoolError::TransactionValidationError(error) => Ok(
                MempoolErrorJava::TransactionValidationError(format!("{:?}", error)),
            ),
        }
    }
}

pub fn export_extern_functions() {}
