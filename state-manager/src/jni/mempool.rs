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

use crate::jni::dtos::JavaStructure;
use jni::objects::{JClass, JObject};
use jni::sys::jbyteArray;
use jni::JNIEnv;
use sbor::{Decode, Encode, TypeId};

use crate::jni::state_manager::JNIStateManager;
use crate::jni::utils::*;
use crate::mempool::*;
use crate::result::{
    ResultStateManagerMaps, StateManagerError, StateManagerResult, ERRCODE_INTERFACE_CASTS,
};
use crate::types::Transaction;

//
// Temporary Structures. This will be removed once we have the final Rust/Java interface.
//

#[derive(Encode, Decode, TypeId)]
struct GetTransactionArgs {
    count: u32,
    prepared_transactions: Vec<Transaction>,
}

impl JavaStructure for GetRelayTransactionsArgs {}

#[derive(Encode, Decode, TypeId)]
struct GetRelayTransactionsArgs {
    initial_delay_millis: u64,
    repeat_delay_millis: u64,
}

impl JavaStructure for GetTransactionArgs {}

//
// JNI Interface
//

#[no_mangle]
extern "system" fn Java_com_radixdlt_mempool_RustMempool_add(
    env: JNIEnv,
    _class: JClass,
    j_state: JObject,
    j_payload: jbyteArray,
) -> jbyteArray {
    let ret = do_add(&env, j_state, j_payload).to_java();

    jni_slice_to_jbytearray(&env, &ret)
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_mempool_RustMempool_getTransactionsForProposal(
    env: JNIEnv,
    _class: JClass,
    j_state: JObject,
    j_payload: jbyteArray,
) -> jbyteArray {
    let ret = do_get_transactions_for_proposal(&env, j_state, j_payload).to_java();

    jni_slice_to_jbytearray(&env, &ret)
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_mempool_RustMempool_handleTransactionsCommitted(
    env: JNIEnv,
    _class: JClass,
    j_state: JObject,
    j_payload: jbyteArray,
) -> jbyteArray {
    let ret = do_handle_transactions_committed(&env, j_state, j_payload).to_java();

    jni_slice_to_jbytearray(&env, &ret)
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_mempool_RustMempool_getCount(
    env: JNIEnv,
    _class: JClass,
    j_state: JObject,
) -> jbyteArray {
    let ret = do_get_count(&env, j_state).to_java();

    jni_slice_to_jbytearray(&env, &ret)
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_mempool_RustMempool_getTransactionsToRelay(
    env: JNIEnv,
    _class: JClass,
    j_state: JObject,
    j_payload: jbyteArray,
) -> jbyteArray {
    let ret = do_get_transactions_to_relay(&env, j_state, j_payload).to_java();

    jni_slice_to_jbytearray(&env, &ret)
}

//
// JNI -> Rust
//

fn do_add(
    env: &JNIEnv,
    j_state: JObject,
    j_payload: jbyteArray,
) -> StateManagerResult<Result<Transaction, MempoolErrorJava>> {
    let state_manager = JNIStateManager::get_state_manager(env, j_state);
    let request_payload: Vec<u8> = jni_jbytearray_to_vector(env, j_payload)?;
    let transaction = Transaction::from_java(&request_payload)?;

    let result = state_manager.mempool.lock().unwrap().add(transaction);

    let mapped_result = result.map_err_sm(|err| err.into())?;
    Ok(mapped_result)
}

fn do_get_transactions_for_proposal(
    env: &JNIEnv,
    j_state: JObject,
    j_payload: jbyteArray,
) -> StateManagerResult<Result<Vec<Transaction>, MempoolErrorJava>> {
    let state_manager = JNIStateManager::get_state_manager(env, j_state);
    let request_payload: Vec<u8> = jni_jbytearray_to_vector(env, j_payload)?;
    let args = GetTransactionArgs::from_java(&request_payload)?;

    let result = state_manager
        .mempool
        .lock()
        .unwrap()
        .get_transactions_for_proposal(args.count.into(), &args.prepared_transactions);

    let mapped_result = result.map_err_sm(|err| err.into())?;
    Ok(mapped_result)
}

fn do_handle_transactions_committed(
    env: &JNIEnv,
    j_state: JObject,
    j_payload: jbyteArray,
) -> StateManagerResult<Result<Vec<Transaction>, MempoolErrorJava>> {
    let state_manager = JNIStateManager::get_state_manager(env, j_state);
    let request_payload: Vec<u8> = jni_jbytearray_to_vector(env, j_payload)?;
    let transactions = Vec::<Transaction>::from_java(&request_payload)?;

    let result = state_manager
        .mempool
        .lock()
        .unwrap()
        .handle_committed_transactions(&transactions);

    let mapped_result = result.map_err_sm(|err| err.into())?;
    Ok(mapped_result)
}

fn do_get_count(env: &JNIEnv, j_state: JObject) -> StateManagerResult<i32> {
    let state_manager = JNIStateManager::get_state_manager(env, j_state);

    let result: i32 = state_manager
        .mempool
        .lock()
        .unwrap()
        .get_count()
        .try_into()
        .unwrap();

    Ok(result)
}

fn do_get_transactions_to_relay(
    env: &JNIEnv,
    j_state: JObject,
    j_payload: jbyteArray,
) -> StateManagerResult<Result<Vec<Transaction>, MempoolErrorJava>> {
    let state_manager = JNIStateManager::get_state_manager(env, j_state);
    let request_payload: Vec<u8> = jni_jbytearray_to_vector(env, j_payload)?;
    let args = GetRelayTransactionsArgs::from_java(&request_payload)?;
    let result = state_manager
        .mempool
        .lock()
        .unwrap()
        .get_transactions_to_relay(args.initial_delay_millis, args.repeat_delay_millis);

    let mapped_result = result.map_err_sm(|err| err.into())?;
    Ok(mapped_result)
}

#[derive(Debug, PartialEq, TypeId, Encode, Decode)]
enum MempoolErrorJava {
    Full { current_size: i64, max_size: i64 },
    Duplicate,
}

impl JavaStructure for MempoolErrorJava {}

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
        }
    }
}
