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
    let public_key: Vec<u8> = env.convert_byte_array(j_public_key).unwrap();
    let state_manager = Arc::new(StateManager::new(public_key));

    env.set_rust_field(interop_state, JNI_FIELD_NAME, state_manager)
        .unwrap();
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_statemanager_StateManager_cleanup(
    env: JNIEnv,
    _class: JClass,
    interop_state: JObject,
) {
    let state_manager: Arc<StateManager> =
        env.take_rust_field(interop_state, JNI_FIELD_NAME).unwrap();

    drop(state_manager);
}

pub fn jni_state_manager(env: JNIEnv, interop_state: JObject) -> Arc<StateManager> {
    let sm: &Arc<StateManager> = &env.get_rust_field(interop_state, JNI_FIELD_NAME).unwrap();
    Arc::clone(sm)
}
