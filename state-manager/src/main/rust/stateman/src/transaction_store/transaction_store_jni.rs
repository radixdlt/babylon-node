use jni::JNIEnv;
use jni::objects::{JClass, JObject, JValue};
use jni::sys::{jbyteArray, jobject, jlong};
use crate::jni_util::use_rust_jni_obj;
use crate::interop_state::INTEROP_STATE_TRANSACTION_STORE_REF;
use crate::transaction_store::TransactionAndProofStore;

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

#[no_mangle]
extern "system" fn Java_com_radixdlt_transaction_RustTransactionStore_getLastTransactionData(
    env: JNIEnv,
    _class: JClass,
    interop_state: JObject
) -> jobject {
    use_transaction_store(env, interop_state, |transaction_store| {
        match transaction_store.get_last_transaction_data() {
            Some(tx) => {
                let stored_tx_class = env.find_class("com/radixdlt/transaction/StoredTransaction")
                    .expect("Can't get StoredTransaction class");

                    let data =  env.byte_array_from_slice(&tx.transaction_data)
                        .expect("Can't create jbyteArray for transaction data");

                let stored_tx = env.new_object(
                    stored_tx_class, 
                    "(J[B)V", 
                    &[JValue::Long(tx.state_version.try_into().unwrap()), JValue::Object(JObject::from(data))])
                    .expect("Can't instantiate StoredTransaction");  

                *stored_tx
            },
            None => *JObject::null()
        }
    })
}


fn use_transaction_store<F, R>(
    env: JNIEnv,
    interop_state: JObject,
    thunk: F
) -> R where F: FnOnce(&mut TransactionAndProofStore) -> R {
    use_rust_jni_obj(env, interop_state, INTEROP_STATE_TRANSACTION_STORE_REF, thunk)
}
