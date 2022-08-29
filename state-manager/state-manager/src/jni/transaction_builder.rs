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
use crate::result::StateManagerResult;
use crate::transaction_builder::{
    create_1mb_txn_manifest, create_intent_bytes, create_new_account_intent_bytes,
    create_notarized_bytes, create_signed_intent_bytes,
};
use jni::objects::JClass;
use jni::sys::jbyteArray;
use jni::JNIEnv;
use radix_engine::types::scrypto_encode;
use sbor::{Decode, Encode, TypeId};
use scrypto::prelude::{EcdsaPublicKey, EcdsaSignature, NetworkDefinition};
use transaction::manifest::compile;
use transaction::model::{SignedTransactionIntent, TransactionHeader, TransactionIntent};

use super::utils::{jni_static_sbor_call, jni_static_sbor_call_flatten_result};

#[no_mangle]
extern "system" fn Java_com_radixdlt_transaction_TransactionBuilder_compileManifest(
    env: JNIEnv,
    _class: JClass,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_static_sbor_call(env, request_payload, do_compile_manifest)
}

fn do_compile_manifest(args: (NetworkDefinition, String)) -> Result<Vec<u8>, String> {
    let (network, manifest_str) = args;

    compile(&manifest_str, &network)
        .map_err(|err| format!("{:?}", err))
        .map(|manifest| scrypto_encode(&manifest))
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_transaction_TransactionBuilder_newAccountIntent(
    env: JNIEnv,
    _class: JClass,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_static_sbor_call(env, request_payload, do_create_new_account_intent)
}

fn do_create_new_account_intent(args: (NetworkDefinition, EcdsaPublicKey)) -> Vec<u8> {
    let (network_definition, public_key) = args;

    create_new_account_intent_bytes(&network_definition, public_key)
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_transaction_TransactionBuilder_createIntent(
    env: JNIEnv,
    _class: JClass,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_static_sbor_call(env, request_payload, do_create_intent_bytes)
}

// To ensure that any change to TransactionHeader is picked up as a compile error,
// not an SBOR error
#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
struct TransactionHeaderJava {
    pub version: u8,
    pub network_id: u8,
    pub start_epoch_inclusive: u64,
    pub end_epoch_exclusive: u64,
    pub nonce: u64,
    pub notary_public_key: EcdsaPublicKey,
    pub notary_as_signatory: bool,
    pub cost_unit_limit: u32,
    pub tip_percentage: u32,
}

impl From<TransactionHeaderJava> for TransactionHeader {
    fn from(header: TransactionHeaderJava) -> Self {
        TransactionHeader {
            version: header.version,
            network_id: header.network_id,
            start_epoch_inclusive: header.start_epoch_inclusive,
            end_epoch_exclusive: header.end_epoch_exclusive,
            nonce: header.nonce,
            notary_public_key: header.notary_public_key,
            notary_as_signatory: header.notary_as_signatory,
            cost_unit_limit: header.cost_unit_limit,
            tip_percentage: header.tip_percentage,
        }
    }
}

fn do_create_intent_bytes(
    args: (NetworkDefinition, TransactionHeaderJava, String),
) -> Result<Vec<u8>, String> {
    let (network_definition, header, manifest) = args;

    create_intent_bytes(&network_definition, header.into(), manifest)
        .map_err(|err| format!("{:?}", err))
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_transaction_TransactionBuilder_build1MBManifest(
    env: JNIEnv,
    _class: JClass,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_static_sbor_call(env, request_payload, do_build_1mb_manifest)
}

fn do_build_1mb_manifest(args: (NetworkDefinition, EcdsaPublicKey)) -> Vec<u8> {
    create_1mb_txn_manifest(args.0, args.1)
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_transaction_TransactionBuilder_createSignedIntentBytes(
    env: JNIEnv,
    _class: JClass,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_static_sbor_call_flatten_result(env, request_payload, do_create_signed_intent_bytes)
}

fn do_create_signed_intent_bytes(
    args: (Vec<u8>, Vec<(EcdsaPublicKey, EcdsaSignature)>),
) -> StateManagerResult<Vec<u8>> {
    let (intent_bytes, signatures) = args;

    // It's passed through to us as bytes - and need to decode these bytes
    let intent = TransactionIntent::from_java(&intent_bytes)?;

    Ok(create_signed_intent_bytes(intent, signatures))
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_transaction_TransactionBuilder_createNotarizedBytes(
    env: JNIEnv,
    _class: JClass,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_static_sbor_call_flatten_result(env, request_payload, do_create_notarized_bytes)
}

fn do_create_notarized_bytes(args: (Vec<u8>, EcdsaSignature)) -> StateManagerResult<Vec<u8>> {
    let (signed_intent_bytes, signature) = args;

    // It's passed through to us as bytes - and need to decode these bytes
    let signed_intent = SignedTransactionIntent::from_java(&signed_intent_bytes)?;

    Ok(create_notarized_bytes(signed_intent, signature))
}

pub fn export_extern_functions() {}
