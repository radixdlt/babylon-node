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
use crate::jni::utils::{jni_jbytearray_to_vector, jni_slice_to_jbytearray};
use crate::result::StateManagerResult;
use crate::transaction_builder::{
    create_new_account_unsigned_manifest, create_notarized_bytes, create_signed_intent_bytes,
};
use jni::objects::JClass;
use jni::sys::jbyteArray;
use jni::JNIEnv;
use sbor::encode_with_type;
use scrypto::buffer::scrypto_decode;
use scrypto::crypto::{EcdsaPublicKey, EcdsaSignature};
use transaction::model::{SignedTransactionIntent, TransactionIntent};

#[no_mangle]
extern "system" fn Java_com_radixdlt_transaction_TransactionBuilder_newAccountManifest(
    env: JNIEnv,
    _class: JClass,
    j_payload: jbyteArray,
) -> jbyteArray {
    let request_payload: Vec<u8> = jni_jbytearray_to_vector(&env, j_payload).unwrap();
    let public_key = EcdsaPublicKey::from_java(&request_payload).unwrap();

    let unsigned_manifest = create_new_account_unsigned_manifest(public_key);
    let result: StateManagerResult<Vec<u8>> = Ok(unsigned_manifest);
    let encoded = encode_with_type(&result);
    jni_slice_to_jbytearray(&env, &encoded)
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_transaction_TransactionBuilder_createSignedIntentBytes(
    env: JNIEnv,
    _class: JClass,
    manifest: jbyteArray,
    public_key: jbyteArray,
    signature: jbyteArray,
) -> jbyteArray {
    let manifest: Vec<u8> = jni_jbytearray_to_vector(&env, manifest).unwrap();
    let intent: TransactionIntent = scrypto_decode(manifest.as_slice()).unwrap();

    let encoded_public_key: Vec<u8> = jni_jbytearray_to_vector(&env, public_key).unwrap();
    let public_key = EcdsaPublicKey::from_java(&encoded_public_key).unwrap();

    let encoded_signature: Vec<u8> = jni_jbytearray_to_vector(&env, signature).unwrap();
    let signature = EcdsaSignature::from_java(&encoded_signature).expect("Invalid signature");

    let signed_manifest = create_signed_intent_bytes(intent, public_key, signature);
    let result: StateManagerResult<Vec<u8>> = Ok(signed_manifest);
    let encoded = encode_with_type(&result);
    jni_slice_to_jbytearray(&env, &encoded)
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_transaction_TransactionBuilder_createNotarizedBytes(
    env: JNIEnv,
    _class: JClass,
    signed_intent: jbyteArray,
    signature: jbyteArray,
) -> jbyteArray {
    let signed_intent: Vec<u8> = jni_jbytearray_to_vector(&env, signed_intent).unwrap();
    let signed_intent: SignedTransactionIntent = scrypto_decode(signed_intent.as_slice()).unwrap();

    let encoded_signature: Vec<u8> = jni_jbytearray_to_vector(&env, signature).unwrap();
    let signature = EcdsaSignature::from_java(&encoded_signature).expect("Invalid signature");

    let notarized_transaction = create_notarized_bytes(signed_intent, signature);
    let result: StateManagerResult<Vec<u8>> = Ok(notarized_transaction);
    let encoded = encode_with_type(&result);
    jni_slice_to_jbytearray(&env, &encoded)
}
