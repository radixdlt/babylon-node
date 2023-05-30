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

use crate::transaction::*;
use jni::objects::JClass;
use jni::sys::jbyteArray;
use jni::JNIEnv;
use node_common::java::*;
use radix_engine::types::PublicKey;
use radix_engine_common::types::Epoch;
use radix_engine_interface::network::NetworkDefinition;
use radix_engine_interface::*;
use transaction::manifest::compile;
use transaction::model::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
struct PrepareIntentRequest {
    network_definition: NetworkDefinition,
    header: TransactionHeaderJava,
    manifest: String,
    blobs: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
struct PrepareIntentResponse {
    intent_bytes: Vec<u8>,
    intent_hash: IntentHash,
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_transaction_TransactionPreparer_prepareIntent(
    env: JNIEnv,
    _class: JClass,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(&env, request_payload, |request: PrepareIntentRequest| -> Result<PrepareIntentResponse, StringError> {
        let manifest = compile(&request.manifest, &request.network_definition, request.blobs)?;

        let (instructions, blobs) = manifest.for_intent();
        let intent = IntentV1 {
            header: request.header.into(),
            instructions,
            blobs,
            attachments: AttachmentsV1 {},
        };

        let prepared_intent = intent.prepare()?;
        
        Ok(PrepareIntentResponse {
            intent_bytes: intent.to_payload_bytes()?,
            intent_hash: prepared_intent.intent_hash(),
        })
    })
}

// We use a separate model to ensure that any change to
// TransactionHeader is picked up as a compile error, not an SBOR error
#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
struct TransactionHeaderJava {
    pub network_id: u8,
    pub start_epoch_inclusive: u64,
    pub end_epoch_exclusive: u64,
    pub nonce: u32,
    pub notary_public_key: PublicKey,
    pub notary_is_signatory: bool,
    pub tip_percentage: u16,
}

impl From<TransactionHeaderJava> for TransactionHeaderV1 {
    fn from(header: TransactionHeaderJava) -> Self {
        TransactionHeaderV1 {
            network_id: header.network_id,
            start_epoch_inclusive: Epoch::of(header.start_epoch_inclusive),
            end_epoch_exclusive: Epoch::of(header.end_epoch_exclusive),
            nonce: header.nonce,
            notary_public_key: header.notary_public_key,
            notary_is_signatory: header.notary_is_signatory,
            tip_percentage: header.tip_percentage,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
struct PrepareSignedIntentRequest {
    intent_bytes: Vec<u8>,
    signatures: Vec<SignatureWithPublicKeyV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
struct PrepareSignedIntentResponse {
    signed_intent_bytes: Vec<u8>,
    intent_hash: IntentHash,
    signed_intent_hash: SignedIntentHash,
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_transaction_TransactionPreparer_prepareSignedIntent(
    env: JNIEnv,
    _class: JClass,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(&env, request_payload, |request: PrepareSignedIntentRequest| -> Result<PrepareSignedIntentResponse, StringError> {
        let signed_intent = SignedIntentV1 {
            intent: IntentV1::from_payload_bytes(&request.intent_bytes)?,
            intent_signatures: IntentSignaturesV1 {
                signatures: request.signatures.into_iter().map(IntentSignatureV1).collect(),
            },
        };

        let prepared_signed_intent = signed_intent.prepare()?;

        Ok(PrepareSignedIntentResponse {
            signed_intent_bytes: signed_intent.to_payload_bytes()?,
            intent_hash: prepared_signed_intent.intent_hash(),
            signed_intent_hash: prepared_signed_intent.signed_intent_hash()
        })
    })
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
struct PrepareNotarizedTransactionRequest {
    signed_intent_bytes: Vec<u8>,
    notary_signature: SignatureV1,
}

// NB - this doesn't need to be decode, because we never receive the hashes from Java
#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode)]
pub struct JavaPreparedNotarizedTransaction {
    pub notarized_transaction_bytes: RawNotarizedTransaction,
    pub intent_hash: IntentHash,
    pub signed_intent_hash: SignedIntentHash,
    pub notarized_transaction_hash: NotarizedTransactionHash,
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_transaction_TransactionPreparer_prepareNotarizedTransaction(
    env: JNIEnv,
    _class: JClass,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(&env, request_payload, |request: PrepareNotarizedTransactionRequest| -> Result<JavaPreparedNotarizedTransaction, StringError> {
        let signed_intent = SignedIntentV1::from_payload_bytes(&request.signed_intent_bytes)?;

        let notarized_transaction = NotarizedTransactionV1 {
            signed_intent,
            notary_signature: NotarySignatureV1(request.notary_signature),
        };

        let prepared = notarized_transaction.prepare()?;

        Ok(JavaPreparedNotarizedTransaction {
            notarized_transaction_bytes: RawNotarizedTransaction(notarized_transaction.to_payload_bytes()?),
            intent_hash: prepared.intent_hash(),
            signed_intent_hash: prepared.signed_intent_hash(),
            notarized_transaction_hash: prepared.notarized_transaction_hash(),
        })
    })
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_transaction_TransactionPreparer_userTransactionToLedger(
    env: JNIEnv,
    _class: JClass,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(&env, request_payload, |payload: RawNotarizedTransaction| -> Result<RawLedgerTransaction, StringError> {
        let notarized_transaction = NotarizedTransactionV1::from_raw(&payload)?;
        Ok(LedgerTransaction::UserV1(Box::new(notarized_transaction)).to_raw()?)
    })
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_transaction_TransactionPreparer_transactionBytesToNotarizedTransactionBytes(
    env: JNIEnv,
    _class: JClass,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(&env, request_payload, |payload: RawLedgerTransaction| -> Result<Option<RawNotarizedTransaction>, StringError> {
        let transaction = LedgerTransaction::from_raw(&payload)?;
        Ok(match transaction {
            LedgerTransaction::UserV1(notarized_transaction) => {
                Some(notarized_transaction.to_raw()?)
            }
            LedgerTransaction::RoundUpdateV1(..) => None,
            LedgerTransaction::Genesis(..) => None,
        })
    })
}

pub fn export_extern_functions() {}
