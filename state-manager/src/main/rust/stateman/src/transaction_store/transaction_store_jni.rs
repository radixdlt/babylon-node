use jni::JNIEnv;
use jni::objects::{JClass, JObject};
use jni::sys::{jbyteArray, jlong};
use crate::jni_util::use_rust_jni_obj;
use crate::interop_state::INTEROP_STATE_TRANSACTION_STORE_REF;
use crate::transaction_store::TransactionStore;

#[no_mangle]
extern "system" fn Java_com_radixdlt_transaction_RustTransactionStore_insertTransaction(
    env: JNIEnv,
    _class: JClass,
    interop_state: JObject,
    j_state_version: jlong,
    j_transaction_bytes: jbyteArray
) {
    use_transaction_store(env, interop_state, |transaction_store| {
        let transaction_bytes: Vec<u8> = env.convert_byte_array(j_transaction_bytes)
            .expect("Can't convert transaction data byte array to vec");
        transaction_store.insert_transaction(j_state_version as u64, transaction_bytes);
    });
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_transaction_RustTransactionStore_getTransactionAtStateVersion(
    env: JNIEnv,
    _class: JClass,
    interop_state: JObject,
    j_state_version: jlong
) -> jbyteArray {
    use_transaction_store(env, interop_state, |transaction_store| {
        let transaction_data = transaction_store.get_transaction(j_state_version as u64);
        env.byte_array_from_slice(&transaction_data)
            .expect("Can't create jbyteArray for transaction data")
    })
}

fn use_transaction_store<F, R>(
    env: JNIEnv,
    interop_state: JObject,
    thunk: F
) -> R where F: FnOnce(&mut TransactionStore) -> R {
    use_rust_jni_obj(env, interop_state, INTEROP_STATE_TRANSACTION_STORE_REF, thunk)
}
