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

use crate::jni_prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
struct PrepareIntentRequest {
    network_definition: NetworkDefinition,
    header: TransactionHeaderJava,
    manifest: String,
    blobs: Vec<Vec<u8>>,
    message: Option<TransactionMessageJava>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
struct PrepareIntentResponse {
    intent_bytes: Vec<u8>,
    intent_hash: TransactionIntentHash,
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_transaction_TransactionPreparer_prepareIntent(
    env: JNIEnv,
    _class: JClass,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(
        &env,
        request_payload,
        |request: PrepareIntentRequest| -> Result<PrepareIntentResponse, StringError> {
            let manifest = compile(
                &request.manifest,
                &request.network_definition,
                BlobProvider::new_with_blobs(request.blobs),
            )?;

            let (instructions, blobs) = manifest.for_intent();
            let intent = IntentV1 {
                header: request.header.into(),
                instructions,
                blobs,
                message: request
                    .message
                    .map(|message| message.into())
                    .unwrap_or_else(|| MessageV1::None),
            };

            let prepared_intent = intent.prepare(&PreparationSettings::latest())?;

            Ok(PrepareIntentResponse {
                intent_bytes: intent.to_raw()?.to_vec(),
                intent_hash: prepared_intent.transaction_intent_hash(),
            })
        },
    )
}

// Yes, this isn't a full transaction specification, but it's good enough for now.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
struct PrepareTransactionIntentV2Request {
    network_definition: NetworkDefinition,
    header: TransactionHeaderJava,
    subintent_discriminators: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
struct PrepareTransactionIntentV2Response {
    raw_transaction_intent: RawTransactionIntent,
    transaction_intent_hash: TransactionIntentHash,
    subintent_hashes: Vec<SubintentHash>,
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_transaction_TransactionPreparer_prepareTransactionIntentV2(
    env: JNIEnv,
    _class: JClass,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(
        &env,
        request_payload,
        |request: PrepareTransactionIntentV2Request| -> Result<PrepareTransactionIntentV2Response, StringError> {
            let PrepareTransactionIntentV2Request {
                network_definition,
                header,
                subintent_discriminators,
            } = request;

            let mut subintent_hashes = vec![];
            let mut subintent_names = vec![];

            let mut transaction_builder = TransactionV2Builder::new()
                .transaction_header(TransactionHeaderV2 {
                    notary_public_key: header.notary_public_key,
                    notary_is_signatory: header.notary_is_signatory,
                    tip_basis_points: (header.tip_percentage as u32) * 100,
                })
                .intent_header(IntentHeaderV2 {
                    network_id: network_definition.id,
                    start_epoch_inclusive: Epoch::of(header.start_epoch_inclusive),
                    end_epoch_exclusive: Epoch::of(header.end_epoch_exclusive),
                    min_proposer_timestamp_inclusive: None,
                    max_proposer_timestamp_exclusive: None,
                    intent_discriminator: header.nonce as u64,
                });

            for subintent_discriminator in subintent_discriminators {
                let mut subintent_builder: PartialTransactionV2Builder = PartialTransactionV2Builder::new()
                    .intent_header(IntentHeaderV2 {
                        network_id: network_definition.id,
                        start_epoch_inclusive: Epoch::of(header.start_epoch_inclusive),
                        end_epoch_exclusive: Epoch::of(header.end_epoch_exclusive),
                        min_proposer_timestamp_inclusive: None,
                        max_proposer_timestamp_exclusive: None,
                        intent_discriminator: subintent_discriminator,
                    })
                    .manifest_builder(|builder| {
                        builder
                            .yield_to_parent(())
                    });
                let child_name = format!("child-{subintent_discriminator}");
                subintent_hashes.push(subintent_builder.subintent_hash());
                subintent_names.push(child_name.clone());
                transaction_builder = transaction_builder.add_signed_child(
                    child_name,
                    subintent_builder.build(),
                );
            }

            transaction_builder = transaction_builder
                .manifest_builder(move |builder| {
                    let mut builder = builder.lock_fee_from_faucet();

                    for child in subintent_names {
                        builder = builder.yield_to_child(child, ());
                    }

                    builder
                });

            let raw_transaction_intent = transaction_builder.create_intent_and_subintent_info()
                .to_raw()
                .unwrap();
            let transaction_intent_hash = transaction_builder.intent_hash();

            Ok(PrepareTransactionIntentV2Response {
                raw_transaction_intent,
                transaction_intent_hash,
                subintent_hashes,
            })
        },
    )
}

// We use a separate model to ensure that any change to
// TransactionHeader is picked up as a compile error, not an SBOR error
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
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

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
enum TransactionMessageJava {
    Plaintext(PlaintextMessageJava),
    Encrypted(EncryptedMessageJava),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
struct PlaintextMessageJava {
    mime_type: String,
    content: MessageContentJava,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
enum MessageContentJava {
    String(String),
    Bytes(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
struct EncryptedMessageJava {
    aes_gcm_payload: Vec<u8>,
    curve_decryptor_sets: Vec<CurveDecryptorSetJava>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
struct CurveDecryptorSetJava {
    dh_ephemeral_public_key: PublicKey,
    decryptors: Vec<DecryptorJava>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
struct DecryptorJava {
    public_key_fingerprint: Vec<u8>,
    aes_wrapped_key: Vec<u8>,
}

impl From<TransactionMessageJava> for MessageV1 {
    fn from(message: TransactionMessageJava) -> Self {
        match message {
            TransactionMessageJava::Plaintext(plaintext) => {
                MessageV1::Plaintext(PlaintextMessageV1 {
                    mime_type: plaintext.mime_type,
                    message: match plaintext.content {
                        MessageContentJava::String(string) => MessageContentsV1::String(string),
                        MessageContentJava::Bytes(bytes) => MessageContentsV1::Bytes(bytes),
                    },
                })
            }
            TransactionMessageJava::Encrypted(encrypted) => {
                MessageV1::Encrypted(EncryptedMessageV1 {
                    encrypted: AesGcmPayload(encrypted.aes_gcm_payload),
                    decryptors_by_curve: encrypted
                        .curve_decryptor_sets
                        .into_iter()
                        .map(DecryptorsByCurve::from)
                        .map(|decryptors| (decryptors.curve_type(), decryptors))
                        .collect(),
                })
            }
        }
    }
}

impl From<CurveDecryptorSetJava> for DecryptorsByCurve {
    fn from(decryptor_set: CurveDecryptorSetJava) -> Self {
        let decryptors = decryptor_set
            .decryptors
            .into_iter()
            .map(|decryptor| {
                (
                    PublicKeyFingerprint(decryptor.public_key_fingerprint.try_into().unwrap()),
                    AesWrapped128BitKey(decryptor.aes_wrapped_key.try_into().unwrap()),
                )
            })
            .collect();
        match decryptor_set.dh_ephemeral_public_key {
            PublicKey::Secp256k1(dh_ephemeral_public_key) => DecryptorsByCurve::Secp256k1 {
                dh_ephemeral_public_key,
                decryptors,
            },
            PublicKey::Ed25519(dh_ephemeral_public_key) => DecryptorsByCurve::Ed25519 {
                dh_ephemeral_public_key,
                decryptors,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
struct PrepareSignedIntentRequest {
    intent_bytes: RawTransactionIntent,
    transaction_signatures: Vec<SignatureWithPublicKeyV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
struct PrepareSignedIntentResponse {
    signed_intent_bytes: RawSignedTransactionIntent,
    intent_hash: TransactionIntentHash,
    signed_intent_hash: SignedTransactionIntentHash,
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_transaction_TransactionPreparer_prepareSignedIntent(
    env: JNIEnv,
    _class: JClass,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(
        &env,
        request_payload,
        |request: PrepareSignedIntentRequest| -> Result<PrepareSignedIntentResponse, StringError> {
            let signed_intent = SignedIntentV1 {
                intent: IntentV1::from_raw(&request.intent_bytes)?,
                intent_signatures: IntentSignaturesV1 {
                    signatures: request
                        .transaction_signatures
                        .into_iter()
                        .map(IntentSignatureV1)
                        .collect(),
                },
            };

            let prepared_signed_intent = signed_intent.prepare(&PreparationSettings::latest())?;

            Ok(PrepareSignedIntentResponse {
                signed_intent_bytes: signed_intent.to_raw()?,
                intent_hash: prepared_signed_intent.transaction_intent_hash(),
                signed_intent_hash: prepared_signed_intent.signed_transaction_intent_hash(),
            })
        },
    )
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
struct PrepareSignedTransactionIntentV2Request {
    intent_bytes: RawTransactionIntent,
    transaction_signatures: Vec<SignatureWithPublicKeyV1>,
    subintent_signatures: Vec<Vec<SignatureWithPublicKeyV1>>,
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_transaction_TransactionPreparer_prepareSignedTransactionIntentV2(
    env: JNIEnv,
    _class: JClass,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(
        &env,
        request_payload,
        |request: PrepareSignedTransactionIntentV2Request| -> Result<PrepareSignedIntentResponse, StringError> {
            let signed_intent = SignedTransactionIntentV2 {
                transaction_intent: TransactionIntentV2::from_raw(&request.intent_bytes)?,
                transaction_intent_signatures: IntentSignaturesV2 {
                    signatures: request
                        .transaction_signatures
                        .into_iter()
                        .map(IntentSignatureV1)
                        .collect(),
                },
                non_root_subintent_signatures: NonRootSubintentSignaturesV2 {
                    by_subintent: request.subintent_signatures.into_iter().map(|signatures| {
                        IntentSignaturesV2 {
                            signatures: signatures.into_iter().map(IntentSignatureV1).collect(),
                        }
                    }).collect(),
                },
            };

            let prepared_signed_intent = signed_intent.prepare(&PreparationSettings::latest())?;

            Ok(PrepareSignedIntentResponse {
                signed_intent_bytes: signed_intent.to_raw()?,
                intent_hash: prepared_signed_intent.transaction_intent_hash(),
                signed_intent_hash: prepared_signed_intent.signed_transaction_intent_hash(),
            })
        },
    )
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
struct PrepareNotarizedTransactionRequest {
    signed_intent_bytes: RawSignedTransactionIntent,
    notary_signature: SignatureV1,
}

// NB - this doesn't need to be decode, because we never receive the hashes from Java
#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode)]
pub struct JavaPreparedNotarizedTransaction {
    pub notarized_transaction_bytes: RawNotarizedTransaction,
    pub intent_hash: TransactionIntentHash,
    pub signed_intent_hash: SignedTransactionIntentHash,
    pub notarized_transaction_hash: NotarizedTransactionHash,
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_transaction_TransactionPreparer_prepareNotarizedTransaction(
    env: JNIEnv,
    _class: JClass,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(&env, request_payload, |request: PrepareNotarizedTransactionRequest| -> Result<JavaPreparedNotarizedTransaction, StringError> {
        let signed_intent = SignedIntentV1::from_raw(&request.signed_intent_bytes)?;

        let notarized_transaction = NotarizedTransactionV1 {
            signed_intent,
            notary_signature: NotarySignatureV1(request.notary_signature),
        };

        let prepared = notarized_transaction.prepare(&PreparationSettings::latest())?;

        Ok(JavaPreparedNotarizedTransaction {
            notarized_transaction_bytes: notarized_transaction.to_raw()?,
            intent_hash: prepared.transaction_intent_hash(),
            signed_intent_hash: prepared.signed_transaction_intent_hash(),
            notarized_transaction_hash: prepared.notarized_transaction_hash(),
        })
    })
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_transaction_TransactionPreparer_prepareNotarizedTransactionV2(
    env: JNIEnv,
    _class: JClass,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(&env, request_payload, |request: PrepareNotarizedTransactionRequest| -> Result<JavaPreparedNotarizedTransaction, StringError> {
        let signed_transaction_intent = SignedTransactionIntentV2::from_raw(&request.signed_intent_bytes)?;

        let notarized_transaction = NotarizedTransactionV2 {
            signed_transaction_intent,
            notary_signature: NotarySignatureV2(request.notary_signature),
        };

        let prepared = notarized_transaction.prepare(&PreparationSettings::latest())?;

        Ok(JavaPreparedNotarizedTransaction {
            notarized_transaction_bytes: notarized_transaction.to_raw()?,
            intent_hash: prepared.transaction_intent_hash(),
            signed_intent_hash: prepared.signed_transaction_intent_hash(),
            notarized_transaction_hash: prepared.notarized_transaction_hash(),
        })
    })
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
struct PrepareUnsigngedPreviewTransactionV2Request {
    raw_transaction_intent: RawTransactionIntent,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode)]
pub struct JavaPreparedPreviewTransactionV2 {
    pub raw_preview_transaction: RawPreviewTransaction,
    pub transaction_intent_hash: TransactionIntentHash,
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_transaction_TransactionPreparer_prepareUnsignedPreviewTransactionV2(
    env: JNIEnv,
    _class: JClass,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(&env, request_payload, |request: PrepareUnsigngedPreviewTransactionV2Request| -> Result<JavaPreparedPreviewTransactionV2, StringError> {
        let transaction_intent = TransactionIntentV2::from_raw(&request.raw_transaction_intent)?;

        let subintent_count = transaction_intent.non_root_subintents.0.len();

        let preview_transaction = PreviewTransactionV2 {
            transaction_intent,
            root_signer_public_keys: Default::default(),
            non_root_subintent_signer_public_keys: (0..subintent_count).map(|_| Default::default()).collect(),
        };

        let prepared = preview_transaction.prepare(&PreparationSettings::latest())?;

        Ok(JavaPreparedPreviewTransactionV2 {
            raw_preview_transaction: preview_transaction.to_raw()?,
            transaction_intent_hash: prepared.transaction_intent.transaction_intent_hash(),
        })
    })
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_transaction_TransactionPreparer_userTransactionToLedger(
    env: JNIEnv,
    _class: JClass,
    request_payload: jbyteArray,
) -> jbyteArray {
    jni_sbor_coded_call(
        &env,
        request_payload,
        |payload: RawNotarizedTransaction| -> Result<RawLedgerTransaction, StringError> {
            let ledger_transaction = LedgerTransaction::from(payload.into_typed()?);
            Ok(ledger_transaction.to_raw()?)
        },
    )
}

pub fn export_extern_functions() {}
