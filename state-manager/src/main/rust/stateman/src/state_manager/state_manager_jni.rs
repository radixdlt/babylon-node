use std::sync::Arc;
use jni::JNIEnv;
use jni::objects::{JClass, JObject};
use jni::sys::jbyteArray;
use crate::state_manager::StateManager;

const STATE_MANAGER_JNI_FIELD_NAME: &str = "stateManager";

#[no_mangle]
extern "system" fn Java_com_radixdlt_statemanager_StateManager_init(
    env: JNIEnv,
    _class: JClass,
    interop_state: JObject,
    j_public_key: jbyteArray
) {
  let public_key: Vec<u8> = env.convert_byte_array(j_public_key).unwrap();
  let state_manager = Arc::new(StateManager::new(public_key));
  
  env.set_rust_field(
    interop_state,
    STATE_MANAGER_JNI_FIELD_NAME,
    state_manager
  ).unwrap();
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_statemanager_StateManager_cleanup(
  env: JNIEnv,
  _class: JClass,
  interop_state: JObject
) {
  let state_manager: Arc<StateManager> =
    env.take_rust_field(
      interop_state,
      STATE_MANAGER_JNI_FIELD_NAME
    ).unwrap();

  drop(state_manager);
}

pub fn get_state_manager_from_jni_env(
  env: JNIEnv,
  interop_state: JObject
) -> Arc<StateManager> {
  env.get_rust_field::<JObject, &str, Arc<StateManager>>(
    interop_state,
    STATE_MANAGER_JNI_FIELD_NAME
  ).unwrap().clone()
}
