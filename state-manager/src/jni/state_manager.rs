use crate::state_manager::StateManager;
use crate::mempool::mock::MockMempool;
use crate::transaction_store::TransactionStore;
use crate::vertex_store::VertexStore;
use jni::objects::{JClass, JObject};
use jni::sys::jlong;
use jni::JNIEnv;
use std::sync::Arc;

const POINTER_JNI_FIELD_NAME: &str = "stateManagerPointer";

#[no_mangle]
extern "system" fn Java_com_radixdlt_statemanager_StateManager_init(
    env: JNIEnv,
    _class: JClass,
    interop_state: JObject,
    j_mempool_size: jlong,
) {
    JNIStateManager::init(&env, interop_state, j_mempool_size);
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_statemanager_StateManager_cleanup(
    env: JNIEnv,
    _class: JClass,
    interop_state: JObject,
) {
    JNIStateManager::cleanup(&env, interop_state);
}

pub struct JNIStateManager {
    state_manager: Arc<StateManager<MockMempool>>,
}

impl JNIStateManager {
    pub fn init(env: &JNIEnv, interop_state: JObject, j_mempool_size: jlong) {
        // Build the basic subcomponents.
        let mempool = MockMempool::new(j_mempool_size.try_into().unwrap()); // XXX: Very Wrong. Should return an error in case it's negative
        let vertex_store = VertexStore::new();
        let transaction_store = TransactionStore::new();

        // Build the state manager.
        let state_manager = Arc::new(StateManager::new(mempool, vertex_store, transaction_store));

        let jni_state_manager = JNIStateManager { state_manager };

        env.set_rust_field(interop_state, POINTER_JNI_FIELD_NAME, jni_state_manager)
            .unwrap();
    }

    pub fn cleanup(env: &JNIEnv, interop_state: JObject) {
        let jni_state_manager: JNIStateManager = env.take_rust_field(interop_state, POINTER_JNI_FIELD_NAME).unwrap();
        drop(jni_state_manager);
    }

    pub fn get_state_manager(
        env: &JNIEnv,
        interop_state: JObject,
    ) -> Arc<StateManager<MockMempool>> {
        let jni_state_manager: &JNIStateManager = &env.get_rust_field(interop_state, POINTER_JNI_FIELD_NAME).unwrap();
        Arc::clone(&jni_state_manager.state_manager)
    }
}
