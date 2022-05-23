mod jni_util;
mod interop_state;
mod transaction_store;
mod vertex_store;

use jni::JNIEnv;
use jni::objects::{JClass, JObject};
use jni::sys::jbyteArray;
use crate::interop_state::*;
use crate::transaction_store::TransactionAndProofStore;
use crate::vertex_store::VertexStore;

#[no_mangle]
extern "system" fn Java_com_radixdlt_statemanager_StateManager_init(
    env: JNIEnv,
    _class: JClass,
    interop_state: JObject
) {
    let vertex_store = VertexStore::new();
    env.set_rust_field(interop_state, INTEROP_STATE_VERTEX_STORE_REF, vertex_store)
        .expect("Can't put VertexStore into JNI env");

    let transaction_store = TransactionAndProofStore::new();
    env.set_rust_field(interop_state, INTEROP_STATE_TRANSACTION_STORE_REF, transaction_store)
        .expect("Can't put TransactionStore into JNI env");
}

// TODO: Just a demo for getting non-ref data from the interop state, delete at some point
#[no_mangle]
extern "system" fn Java_com_radixdlt_statemanager_StateManager_getPublicKey(
    env: JNIEnv,
    _class: JClass,
    interop_state: JObject
) -> jbyteArray {
    let node_info = get_node_info(env, interop_state);
    env.byte_array_from_slice(&node_info.public_key)
        .expect("Can't create jbyteArray for public key")
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_statemanager_StateManager_cleanup(
    env: JNIEnv,
    _class: JClass,
    interop_state: JObject
) {
    let vertex_store: VertexStore = env.take_rust_field(interop_state, INTEROP_STATE_VERTEX_STORE_REF)
        .expect("Can't take VertexStore from JNI env");
    drop(vertex_store);

    let transaction_store: TransactionAndProofStore = env.take_rust_field(interop_state, INTEROP_STATE_TRANSACTION_STORE_REF)
        .expect("Can't take TransactionStore from JNI env");
    drop(transaction_store);
}
