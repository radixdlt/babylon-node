use crate::jni::state_manager::jni_state_manager;
use jni::objects::{JClass, JObject};
use jni::sys::{jbyteArray, jlong};
use jni::JNIEnv;

#[no_mangle]
extern "system" fn Java_com_radixdlt_transaction_RustTransactionStore_insertTransaction(
    env: JNIEnv,
    _class: JClass,
    interop_state: JObject,
    j_state_version: jlong,
    j_transaction_bytes: jbyteArray,
) {
    let state_manager = jni_state_manager(env, interop_state);

    let transaction_bytes: Vec<u8> = env
        .convert_byte_array(j_transaction_bytes)
        .expect("Can't convert transaction data byte array to vec");

    // Only get the lock for transaction store
    state_manager
        .transaction_store
        .lock()
        .unwrap()
        .insert_transaction(j_state_version as u64, transaction_bytes);
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_transaction_RustTransactionStore_getTransactionAtStateVersion(
    env: JNIEnv,
    _class: JClass,
    interop_state: JObject,
    j_state_version: jlong,
) -> jbyteArray {
    let state_manager = jni_state_manager(env, interop_state);

    // Only get the lock for transaction store
    let transaction_store = state_manager.transaction_store.lock().unwrap();

    let transaction_data = transaction_store.get_transaction(j_state_version as u64);

    env.byte_array_from_slice(transaction_data)
        .expect("Can't create jbyteArray for transaction data")
}
