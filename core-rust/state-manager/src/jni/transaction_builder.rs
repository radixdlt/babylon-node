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

use crate::jni::java_structure::JavaStructure;
use crate::result::StateManagerResult;
use crate::transaction::{
    create_genesis_ledger_transaction_bytes, create_intent_bytes, create_manifest,
    create_new_account_intent_bytes, create_notarized_bytes, create_signed_intent_bytes,
    LedgerTransaction,
};
use jni::objects::JClass;
use jni::sys::jbyteArray;
use jni::JNIEnv;
use radix_engine::types::{
    scrypto_decode, scrypto_encode, Decode, Encode, PublicKey, Signature, SignatureWithPublicKey,
    TypeId,
};
use radix_engine_interface::core::NetworkDefinition;
use radix_engine_interface::crypto::EcdsaSecp256k1PublicKey;
use radix_engine_interface::math::Decimal;
use radix_engine_interface::scrypto;
use std::collections::BTreeMap;
use transaction::model::{
    NotarizedTransaction, SignedTransactionIntent, TransactionHeader, TransactionIntent,
};

use super::utils::{jni_static_sbor_call, jni_static_sbor_call_flatten_result};

#[no_mangle]
extern "system" fn Java_com_radixdlt_transaction_TransactionBuilder_compileManifest(
    env: JNIEnv,
    _class: JClass,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_static_sbor_call(env, request_payload, do_compile_manifest)
}

fn do_compile_manifest(
    (network, manifest_str, blobs): (NetworkDefinition, String, Vec<Vec<u8>>),
) -> Result<Vec<u8>, String> {
    create_manifest(&network, &manifest_str, blobs)
        .map_err(|err| format!("{:?}", err))
        .map(|manifest| scrypto_encode(&manifest).unwrap())
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_transaction_TransactionBuilder_createGenesisLedgerTransaction(
    env: JNIEnv,
    _class: JClass,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_static_sbor_call(env, request_payload, do_create_genesis_ledger_transaction)
}

fn do_create_genesis_ledger_transaction(
    (validator_set, initial_epoch, rounds_per_epoch, num_unstake_epochs): (
        BTreeMap<EcdsaSecp256k1PublicKey, Decimal>,
        u64,
        u64,
        u64,
    ),
) -> Vec<u8> {
    create_genesis_ledger_transaction_bytes(
        validator_set,
        initial_epoch,
        rounds_per_epoch,
        num_unstake_epochs,
    )
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_transaction_TransactionBuilder_newAccountIntent(
    env: JNIEnv,
    _class: JClass,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_static_sbor_call(env, request_payload, do_create_new_account_intent)
}

fn do_create_new_account_intent(
    (network_definition, public_key): (NetworkDefinition, PublicKey),
) -> Vec<u8> {
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
#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
struct TransactionHeaderJava {
    pub version: u8,
    pub network_id: u8,
    pub start_epoch_inclusive: u64,
    pub end_epoch_exclusive: u64,
    pub nonce: u64,
    pub notary_public_key: PublicKey,
    pub notary_as_signatory: bool,
    pub cost_unit_limit: u32,
    pub tip_percentage: u16,
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
    (network_definition, header, manifest, blobs): (
        NetworkDefinition,
        TransactionHeaderJava,
        String,
        Vec<Vec<u8>>,
    ),
) -> Result<Vec<u8>, String> {
    create_intent_bytes(&network_definition, header.into(), manifest, blobs)
        .map_err(|err| format!("{:?}", err))
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
    (intent_bytes, signatures): (Vec<u8>, Vec<SignatureWithPublicKey>),
) -> StateManagerResult<Vec<u8>> {
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

fn do_create_notarized_bytes(
    (signed_intent_bytes, signature): (Vec<u8>, Signature),
) -> StateManagerResult<Vec<u8>> {
    // It's passed through to us as bytes - and need to decode these bytes
    let signed_intent = SignedTransactionIntent::from_java(&signed_intent_bytes)?;

    Ok(create_notarized_bytes(signed_intent, signature))
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_transaction_TransactionBuilder_userTransactionToLedger(
    env: JNIEnv,
    _class: JClass,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_static_sbor_call_flatten_result(env, request_payload, do_user_transaction_to_ledger)
}

fn do_user_transaction_to_ledger(args: Vec<u8>) -> StateManagerResult<Vec<u8>> {
    let notarized_transaction: NotarizedTransaction = scrypto_decode(&args)?;
    Ok(scrypto_encode(&LedgerTransaction::User(
        notarized_transaction,
    ))?)
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_transaction_TransactionBuilder_transactionBytesToNotarizedTransactionBytes(
    env: JNIEnv,
    _class: JClass,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_static_sbor_call_flatten_result(
        env,
        request_payload,
        do_transaction_bytes_to_notarized_transaction_bytes,
    )
}

fn do_transaction_bytes_to_notarized_transaction_bytes(
    args: Vec<u8>,
) -> StateManagerResult<Option<Vec<u8>>> {
    let transaction: LedgerTransaction = scrypto_decode(&args)?;
    Ok(match transaction {
        LedgerTransaction::User(notarized_transaction) => Some(notarized_transaction.to_bytes()?),
        LedgerTransaction::Validator(..) => None,
        LedgerTransaction::System(..) => None,
    })
}

pub fn export_extern_functions() {}
