mod transaction_store;

use jni::objects::{JClass, JObject};
use jni::sys::{jbyteArray, jlong};
use jni::JNIEnv;
use transaction_store::*;

const STATE_REF_FIELD_NAME: &str = "value";

struct NodeState {
    public_key: Vec<u8>,
    transaction_store: TransactionStore
}

#[no_mangle]
pub extern "system" fn Java_com_radixdlt_statemanager_StateManagerRustInterop_init(
    env: JNIEnv,
    _class: JClass,
    rust_state_ref: JObject,
    j_public_key: jbyteArray
) {
    let public_key: Vec<u8> = env.convert_byte_array(j_public_key)
        .expect("Can't convert public key byte array to vec");

    let node_state = NodeState {
        public_key: public_key,
        transaction_store: TransactionStore::new()
    };

    env.set_rust_field(rust_state_ref, STATE_REF_FIELD_NAME, node_state)
        .expect("Can't set state ref field on RustStateRef");
}

#[no_mangle]
pub extern "system" fn Java_com_radixdlt_statemanager_StateManagerRustInterop_getPublicKey(
    env: JNIEnv,
    _class: JClass,
    rust_state_ref: JObject
) -> jbyteArray {
    with_node_state(env, rust_state_ref, |node_state| {
        env.byte_array_from_slice(&node_state.public_key)
            .expect("Can't create jbyteArray from public key byte vec")
    })
}

#[no_mangle]
pub extern "system" fn Java_com_radixdlt_statemanager_StateManagerRustInterop_insertTransaction(
    env: JNIEnv,
    _class: JClass,
    rust_state_ref: JObject,
    j_state_version: jlong,
    j_transaction_bytes: jbyteArray
) {
    with_node_state(env, rust_state_ref, |node_state| {
        let transaction_bytes: Vec<u8> = env.convert_byte_array(j_transaction_bytes)
            .expect("Can't convert transaction data byte array to vec");
        node_state.transaction_store.insert_transaction(j_state_version as u64, transaction_bytes);
    });
}

#[no_mangle]
pub extern "system" fn Java_com_radixdlt_statemanager_StateManagerRustInterop_getTransactionAtStateVersion(
    env: JNIEnv,
    _class: JClass,
    rust_state_ref: JObject,
    j_state_version: jlong
) -> jbyteArray {
    with_node_state(env, rust_state_ref, |node_state| {
        let transaction_data = node_state.transaction_store.get_transaction(j_state_version as u64);
        env.byte_array_from_slice(&transaction_data)
            .expect("Can't create jbyteArray for transaction data")
    })
}

fn with_node_state<F, R>(
    env: JNIEnv,
    rust_state_ref: JObject,
    thunk: F
) -> R where F: FnOnce(&mut NodeState) -> R {
    let mut node_state: NodeState = env.take_rust_field(rust_state_ref, STATE_REF_FIELD_NAME)
        .expect("Can't get NodeState ref from JNI env");
    let result = thunk(&mut node_state);
    env.set_rust_field(rust_state_ref, STATE_REF_FIELD_NAME, node_state)
        .expect("Can't set NodeState ref on JNI env");
    result
}
