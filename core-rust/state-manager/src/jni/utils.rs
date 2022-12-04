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

use jni::JNIEnv;
use jni::{objects::JObject, sys::jbyteArray};
use radix_engine::types::{ScryptoDecode, ScryptoEncode, ScryptoTypeId};
use std::ops::DerefMut;

use crate::jni::state_manager::ActualStateManager;
use crate::result::{StateManagerError, StateManagerResult, ERRCODE_JNI};

use super::{java_structure::JavaStructure, state_manager::JNIStateManager};

pub fn jni_jbytearray_to_vector(
    env: &JNIEnv,
    jbytearray: jbyteArray,
) -> StateManagerResult<Vec<u8>> {
    env.convert_byte_array(jbytearray)
        .map_err(|jerr| StateManagerError::create(ERRCODE_JNI, jerr.to_string()))
}

pub fn jni_slice_to_jbytearray(env: &JNIEnv, slice: &[u8]) -> jbyteArray {
    // Unwrap looks bad here, but:
    //
    // 1. by looking at the source code of the JNI, it seems this
    // cannot really fail unless OOM.
    //
    // 2. in case this fails, we would still have to map the error
    // code in a jbyteArray, so possibly the only way to solve this is
    // by having a static bytearray to return in this extremely remote
    // case.
    env.byte_array_from_slice(slice)
        .expect("Can't convert &[u8] back to jbyteArray - likely due to OOM")
}

pub fn jni_static_sbor_call<
    Args: JavaStructure + ScryptoDecode,
    Response: JavaStructure + ScryptoTypeId + ScryptoEncode + ScryptoDecode,
>(
    env: JNIEnv,
    request_payload: jbyteArray,
    method: impl FnOnce(Args) -> Response,
) -> jbyteArray {
    let response_result = jni_static_sbor_call_inner(&env, request_payload, method);
    jni_slice_to_jbytearray(&env, &response_result.to_java().unwrap())
}

#[tracing::instrument(skip_all)]
fn jni_static_sbor_call_inner<Args: JavaStructure, Response: JavaStructure>(
    env: &JNIEnv,
    request_payload: jbyteArray,
    method: impl FnOnce(Args) -> Response,
) -> StateManagerResult<Response> {
    let vec_payload = jni_jbytearray_to_vector(env, request_payload)?;
    let args = Args::from_java(&vec_payload)?;

    let response = method(args);
    Ok(response)
}

pub fn jni_static_sbor_call_flatten_result<
    Args: JavaStructure + ScryptoDecode,
    Response: JavaStructure + ScryptoTypeId + ScryptoDecode + ScryptoEncode,
>(
    env: JNIEnv,
    request_payload: jbyteArray,
    method: impl FnOnce(Args) -> StateManagerResult<Response>,
) -> jbyteArray {
    let response_result = jni_static_sbor_call_flatten_result_inner(&env, request_payload, method);
    jni_slice_to_jbytearray(&env, &response_result.to_java().unwrap())
}

fn jni_static_sbor_call_flatten_result_inner<Args: JavaStructure, Response: JavaStructure>(
    env: &JNIEnv,
    request_payload: jbyteArray,
    method: impl FnOnce(Args) -> StateManagerResult<Response>,
) -> StateManagerResult<Response> {
    let vec_payload = jni_jbytearray_to_vector(env, request_payload)?;
    let args = Args::from_java(&vec_payload)?;

    let response = method(args)?;
    Ok(response)
}

pub fn jni_state_manager_sbor_read_call<
    Args: JavaStructure + ScryptoDecode,
    Response: JavaStructure + ScryptoEncode + ScryptoDecode + ScryptoTypeId,
>(
    env: JNIEnv,
    j_state_manager: JObject,
    request_payload: jbyteArray,
    method: impl FnOnce(&ActualStateManager, Args) -> Response,
) -> jbyteArray {
    let response_result =
        jni_state_manager_sbor_read_call_inner(&env, j_state_manager, request_payload, method);
    jni_slice_to_jbytearray(&env, &response_result.to_java().unwrap())
}

pub fn jni_state_manager_sbor_call<
    Args: JavaStructure + ScryptoDecode,
    Response: JavaStructure + ScryptoEncode + ScryptoDecode + ScryptoTypeId,
>(
    env: JNIEnv,
    j_state_manager: JObject,
    request_payload: jbyteArray,
    method: impl FnOnce(&mut ActualStateManager, Args) -> Response,
) -> jbyteArray {
    let response_result =
        jni_state_manager_sbor_call_inner(&env, j_state_manager, request_payload, method);
    jni_slice_to_jbytearray(&env, &response_result.to_java().unwrap())
}

#[tracing::instrument(skip_all)]
fn jni_state_manager_sbor_read_call_inner<Args: JavaStructure, Response: JavaStructure>(
    env: &JNIEnv,
    j_state_manager: JObject,
    request_payload: jbyteArray,
    method: impl FnOnce(&ActualStateManager, Args) -> Response,
) -> StateManagerResult<Response> {
    let vec_payload = jni_jbytearray_to_vector(env, request_payload)?;
    let args = Args::from_java(&vec_payload)?;

    let state_manager_arc = JNIStateManager::get_state_manager(env, j_state_manager);
    let state_manager = state_manager_arc.read();

    let response = method(&state_manager, args);
    Ok(response)
}

fn jni_state_manager_sbor_call_inner<Args: JavaStructure, Response: JavaStructure>(
    env: &JNIEnv,
    j_state_manager: JObject,
    request_payload: jbyteArray,
    method: impl FnOnce(&mut ActualStateManager, Args) -> Response,
) -> StateManagerResult<Response> {
    let vec_payload = jni_jbytearray_to_vector(env, request_payload)?;
    let args = Args::from_java(&vec_payload)?;

    let state_manager_arc = JNIStateManager::get_state_manager(env, j_state_manager);
    let mut state_manager = state_manager_arc.write();

    let response = method(&mut state_manager, args);
    Ok(response)
}

pub fn jni_state_manager_sbor_call_flatten_result<
    Args: JavaStructure + ScryptoDecode,
    Response: JavaStructure + ScryptoEncode + ScryptoDecode + ScryptoTypeId,
>(
    env: JNIEnv,
    j_state_manager: JObject,
    request_payload: jbyteArray,
    method: impl FnOnce(&mut ActualStateManager, Args) -> StateManagerResult<Response>,
) -> jbyteArray {
    let response_result = jni_state_manager_sbor_call_flatten_result_inner(
        &env,
        j_state_manager,
        request_payload,
        method,
    );
    jni_slice_to_jbytearray(&env, &response_result.to_java().unwrap())
}

fn jni_state_manager_sbor_call_flatten_result_inner<
    Args: JavaStructure,
    Response: JavaStructure,
>(
    env: &JNIEnv,
    j_state_manager: JObject,
    request_payload: jbyteArray,
    method: impl FnOnce(&mut ActualStateManager, Args) -> StateManagerResult<Response>,
) -> StateManagerResult<Response> {
    let vec_payload = jni_jbytearray_to_vector(env, request_payload)?;
    let args = Args::from_java(&vec_payload)?;

    let state_manager_arc = JNIStateManager::get_state_manager(env, j_state_manager);
    let mut state_manager = state_manager_arc.write();

    let response = method(state_manager.deref_mut(), args)?;
    Ok(response)
}
