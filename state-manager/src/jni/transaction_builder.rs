use jni::JNIEnv;
use jni::objects::{JClass};
use jni::sys::{jbyteArray};
use sbor::encode_with_type;
use scrypto::buffer::scrypto_decode;
use scrypto::crypto::{EcdsaPublicKey, EcdsaSignature, sha256};
use transaction::model::{SignedTransactionIntent, TransactionIntent};
use transaction::validation::verify_ecdsa;
use crate::jni::utils::{jni_jbytearray_to_vector, jni_slice_to_jbytearray};
use crate::result::StateManagerResult;
use crate::transaction_builder::{combine, combine_for_notary, create_new_account_unsigned_manifest};

#[no_mangle]
extern "system" fn Java_com_radixdlt_transaction_TransactionBuilder_account(
    env: JNIEnv,
    _class: JClass,
    j_payload: jbyteArray,
) -> jbyteArray {
    let request_payload: Vec<u8> = jni_jbytearray_to_vector(&env, j_payload).unwrap();
    let public_key = EcdsaPublicKey::try_from(request_payload.as_slice()).unwrap();
    let unsigned_manifest = create_new_account_unsigned_manifest(public_key);
    let result: StateManagerResult<Vec<u8>> = Ok(unsigned_manifest);
    let encoded = encode_with_type(&result);
    jni_slice_to_jbytearray(&env, &encoded)
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_transaction_TransactionBuilder_combineForNotary(
    env: JNIEnv,
    _class: JClass,
    manifest: jbyteArray,
    public_key: jbyteArray,
    signature: jbyteArray,
) -> jbyteArray {
    let manifest: Vec<u8> = jni_jbytearray_to_vector(&env, manifest).unwrap();
    let intent: TransactionIntent = scrypto_decode(manifest.as_slice()).unwrap();

    let public_key: Vec<u8> = jni_jbytearray_to_vector(&env, public_key).unwrap();
    let public_key = EcdsaPublicKey::try_from(public_key.as_slice()).unwrap();

    let signature: Vec<u8> = jni_jbytearray_to_vector(&env, signature).unwrap();
    let signature = EcdsaSignature::try_from(signature.as_slice())
        .expect("Invalid signature");

    if !verify_ecdsa(&intent.to_bytes(), &public_key, &signature) {
        let hash = sha256(sha256(intent.to_bytes()));
        panic!("Invalid signature on hash {:?}", hash);
    }

    let signed_manifest = combine_for_notary(intent, public_key, signature);
    let result: StateManagerResult<Vec<u8>> = Ok(signed_manifest);
    let encoded = encode_with_type(&result);
    jni_slice_to_jbytearray(&env, &encoded)
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_transaction_TransactionBuilder_combine(
    env: JNIEnv,
    _class: JClass,
    signed_intent: jbyteArray,
    signature: jbyteArray,
) -> jbyteArray {
    let signed_intent: Vec<u8> = jni_jbytearray_to_vector(&env, signed_intent).unwrap();
    let signed_intent: SignedTransactionIntent = scrypto_decode(signed_intent.as_slice()).unwrap();
    let signature: Vec<u8> = jni_jbytearray_to_vector(&env, signature).unwrap();
    let signature = EcdsaSignature::try_from(signature.as_slice())
        .expect("Invalid signature");
    let notarized_transaction = combine(signed_intent, signature);
    let result: StateManagerResult<Vec<u8>> = Ok(notarized_transaction);
    let encoded = encode_with_type(&result);
    jni_slice_to_jbytearray(&env, &encoded)
}
