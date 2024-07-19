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

use crate::engine_prelude::*;
use jni::sys::jbyteArray;
use jni::JNIEnv;
use std::panic;
use std::panic::AssertUnwindSafe;

use crate::java::structure::{StructFromJava, StructToJava};

use crate::java::result::JavaResult;

pub fn jni_jbytearray_to_vector(env: &JNIEnv, jbytearray: jbyteArray) -> JavaResult<Vec<u8>> {
    Ok(env.convert_byte_array(jbytearray)?)
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

/// A convenience method for a `jni_call()` which uses SBOR codec for its argument and result.
pub fn jni_sbor_coded_call<Args: ScryptoDecode, Response: ScryptoEncode>(
    env: &JNIEnv,
    encoded_request: jbyteArray,
    method: impl FnOnce(Args) -> Response,
) -> jbyteArray {
    jni_sbor_coded_fallible_call(env, encoded_request, |args| Ok(method(args)))
}

/// A convenience method for a `jni_call()` which uses SBOR codec for its argument and its
/// potentially-erroneous result.
pub fn jni_sbor_coded_fallible_call<Args: ScryptoDecode, Response: ScryptoEncode>(
    env: &JNIEnv,
    encoded_request: jbyteArray,
    method: impl FnOnce(Args) -> JavaResult<Response>,
) -> jbyteArray {
    jni_call(env, || {
        let result = jni_jbytearray_to_vector(env, encoded_request)
            .and_then(|bytes| Args::from_java(&bytes))
            .and_then(method);
        jni_slice_to_jbytearray(env, &result.to_java().unwrap())
    })
    .unwrap_or_else(std::ptr::null_mut)
}

/// A convenience method for the case when input should remain in the form of raw byte array
/// (i.e. no SBOR decoding is needed). Output is still properly encoded with SBOR.
pub fn jni_raw_sbor_fallible_call<Response: ScryptoEncode>(
    env: &JNIEnv,
    encoded_request: jbyteArray,
    method: impl FnOnce(Vec<u8>) -> JavaResult<Response>,
) -> jbyteArray {
    jni_call(env, || {
        let result = jni_jbytearray_to_vector(env, encoded_request)
            .and_then(method);
        jni_slice_to_jbytearray(env, &result.to_java().unwrap())
    })
    .unwrap_or_else(std::ptr::null_mut)
}

/// Executes the given function in a way that is panic-safe on a JNI-originating stack.
/// This is achieved by intercepting any panic (i.e. `catch_unwind()`) and throwing a Java-side
/// `RustPanicException` instead.
///
/// *This is a mandatory wrapper for all JNI calls.*
/// Every top-level JNI method MUST immediately enter this template method (i.e. before executing
/// any logic that may panic) - otherwise, unwinding a stack through the JNI boundary results in an
/// undefined behavior.
/// It is fine to enter this method indirectly, e.g. via one of convenience variants defined above.
///
/// Note on the "abrupt return" from this method:
/// In case a Java exception is thrown, this function will return `None`, signalling the caller that
/// there is no valid result of the call, and that it should immediately return from the JNI method,
/// allowing the Java-side JNI infra to continue with the exception.
/// For syntactic reasons, the caller may need to return some value back to the JNI - it is then
/// fine to return virtually any value of a valid type (e.g. a `null_mut()` pointer, which denotes
/// a Java `null`), since it will be ignored by the Java-side JNI infra anyway.
pub fn jni_call<R>(env: &JNIEnv, callable: impl FnOnce() -> R) -> Option<R> {
    let result = panic::catch_unwind(AssertUnwindSafe(callable)).map_err(|panic_payload| {
        if let Some(string) = panic_payload.downcast_ref::<String>() {
            string.clone()
        } else if let Some(str_ref) = panic_payload.downcast_ref::<&'static str>() {
            str_ref.to_string()
        } else {
            format!("[non-string panic payload: {:?}]", panic_payload)
        }
    });
    match result {
        Ok(return_value) => Some(return_value),
        Err(panic_message) => {
            let throw_result =
                env.throw_new("com/radixdlt/exceptions/RustPanicException", panic_message);
            if let Err(throw_error) = throw_result {
                println!("failed to throw a java exception: {:?}", throw_error)
            }
            None
        }
    }
}

/// A type to allow easy transporting of error messages over the boundary, by returning a Result<X, StringError>
///
/// Note - doesn't implement Debug itself, to avoid the blanket impl below from failing
#[derive(Clone, PartialEq, Eq, Sbor)]
#[sbor(transparent)]
pub struct StringError(String);

impl<T: std::fmt::Debug> From<T> for StringError {
    fn from(value: T) -> Self {
        Self(format!("{value:?}"))
    }
}
