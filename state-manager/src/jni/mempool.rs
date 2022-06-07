use jni::objects::{JClass, JObject};
use jni::sys::jbyteArray;
use jni::JNIEnv;

use crate::jni::state_manager::JNIStateManager;
use crate::jni::utils::*;
use crate::mempool::*;
use crate::result::StateManagerResult;
use crate::types::{JavaStructure, Transaction};

#[no_mangle]
extern "system" fn Java_com_radixdlt_mempool_RustMempool_add(
    env: JNIEnv,
    _class: JClass,
    j_state: JObject,
    j_txn: jbyteArray,
) -> jbyteArray {
    let ret = do_add(&env, j_state, j_txn).to_java();

    jni_slice_to_jbytearray(&env, &ret)
}

fn do_add(env: &JNIEnv, j_state: JObject, j_txn: jbyteArray) -> StateManagerResult<()> {
    let state_manager = JNIStateManager::get_state_manager(env, j_state);

    let s_txn: Vec<u8> = jni_jbytearray_to_vector(env, j_txn)?;

    let txn = Transaction::from_java(&s_txn)?;

    let ret = state_manager
        .mempool
        .lock()
        .unwrap()
        .add(txn)
        .map_err(|e| e.to_state_manager_error());

    ret
}
