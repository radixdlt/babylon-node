use jni::JNIEnv;
use jni::objects::{JClass};
use jni::sys::{jbyteArray};
use sbor::encode_with_type;
use scrypto::buffer::scrypto_decode;
use scrypto::crypto::{EcdsaPublicKey, EcdsaSignature};
use transaction::model::{TransactionIntent, TransactionManifest};
use crate::jni::utils::{jni_jbytearray_to_vector, jni_slice_to_jbytearray};
use crate::result::StateManagerResult;
use crate::transaction_builder::{combine_for_notary, create_new_account_unsigned_manifest};

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
extern "system" fn Java_com_radixdlt_transaction_TransactionBuilder_combine(
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
    let signature = EcdsaSignature::try_from(signature.as_slice()).unwrap();
    let signed_manifest = combine_for_notary(intent, public_key, signature);
    let encoded = encode_with_type(&signed_manifest);
    jni_slice_to_jbytearray(&env, &encoded)
}
