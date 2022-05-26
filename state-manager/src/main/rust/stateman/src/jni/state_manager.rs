use crate::state_manager::StateManager;
use jni::objects::{JClass, JObject};
use jni::sys::jbyteArray;
use jni::JNIEnv;
use std::sync::Arc;

const JNI_FIELD_NAME: &str = "stateManager";

#[no_mangle]
extern "system" fn Java_com_radixdlt_statemanager_StateManager_init(
    env: JNIEnv,
    _class: JClass,
    interop_state: JObject,
    j_public_key: jbyteArray,
) {
    JNIStateManager::init(env, interop_state, j_public_key);
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_statemanager_StateManager_cleanup(
    env: JNIEnv,
    _class: JClass,
    interop_state: JObject,
) {
    JNIStateManager::cleanup(env, interop_state);
}

use crate::mempool::mock::MockMempool;
use crate::transaction_store::TransactionStore;
use crate::vertex_store::VertexStore;

pub struct JNIStateManager {
    state_manager: Arc<StateManager<MockMempool>>,
}

impl JNIStateManager {
    pub fn init(env: JNIEnv, interop_state: JObject, j_public_key: jbyteArray) {
        // Build the basic subcomponents.
        let mempool = MockMempool::new(64); // TODO: parameter
        let vtxstore = VertexStore::new();
        let txnstore = TransactionStore::new();

        // Build the state manager.
        let state_manager = Arc::new(StateManager::new(mempool, vtxstore, txnstore));

        let nodesm = JNIStateManager { state_manager };

        env.set_rust_field(interop_state, JNI_FIELD_NAME, nodesm)
            .unwrap();
    }

    pub fn cleanup(env: JNIEnv, interop_state: JObject) {
        let nodesm: JNIStateManager = env.take_rust_field(interop_state, JNI_FIELD_NAME).unwrap();
        drop(nodesm);
    }

    pub fn get_state_manager(
        env: JNIEnv,
        interop_state: JObject,
    ) -> Arc<StateManager<MockMempool>> {
        let nodesm: &JNIStateManager = &env.get_rust_field(interop_state, JNI_FIELD_NAME).unwrap();
        Arc::clone(&nodesm.state_manager)
    }
}
