use std::sync::Arc;
use std::sync::MutexGuard;
use jni::JNIEnv;
use jni::objects::{JClass, JObject};
use jni::sys::jbyteArray;
use tokio::runtime::Runtime as TokioRuntime;
use tokio::sync::mpsc::Sender;
use crate::state_manager::{StateManager, StateManagerRequest, init, blocking_get_state_manager};

const TOKIO_RUNTIME_STATE_FIELD_NAME: &str = "tokioRuntime";
const STATE_MANAGER_CHANNEL_SENDER_STATE_FIELD_NAME: &str = "stateManagerChannelSender";

#[no_mangle]
extern "system" fn Java_com_radixdlt_statemanager_StateManager_init(
    env: JNIEnv,
    _class: JClass,
    interop_state: JObject,
    j_public_key: jbyteArray
) {

  let public_key: Vec<u8> = env.convert_byte_array(j_public_key).unwrap();
  let (tokio_runtime, state_manager_channel_sender) = init(public_key);
  
  env.set_rust_field(
    interop_state,
    TOKIO_RUNTIME_STATE_FIELD_NAME,
    tokio_runtime
  ).unwrap();
    
  env.set_rust_field(
    interop_state,
    STATE_MANAGER_CHANNEL_SENDER_STATE_FIELD_NAME,
    state_manager_channel_sender
  ).unwrap();
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_statemanager_StateManager_cleanup(
  env: JNIEnv,
  _class: JClass,
  interop_state: JObject
) {
  let tokio_runtime: Arc<TokioRuntime> =
    env.take_rust_field(
      interop_state,
      TOKIO_RUNTIME_STATE_FIELD_NAME
    ).unwrap();

    // TODO: tokio_runtime.shutdown_timeout(Duration::from_millis(1000));

  drop(tokio_runtime);

  let state_manager_channel_sender: Sender<StateManagerRequest> =
    env.take_rust_field(
      interop_state,
      STATE_MANAGER_CHANNEL_SENDER_STATE_FIELD_NAME
    ).unwrap();

  drop(state_manager_channel_sender);
}

pub fn blocking_get_state_manager_from_jni(
  env: JNIEnv,
  interop_state: JObject
) -> StateManager {
  let (tokio_runtime, state_manager_channel_sender) =
    get_runtime_and_channel_from_jni_env(env, interop_state);
  blocking_get_state_manager(tokio_runtime, state_manager_channel_sender)
}

fn get_runtime_and_channel_from_jni_env(
  env: JNIEnv,
  interop_state: JObject
) -> (Arc<TokioRuntime>, Sender<StateManagerRequest>) {
  // TODO: is the extra scope required to free the mutex?
  let tokio_runtime: Arc<TokioRuntime> = {
    let tokio_runtime_arc: MutexGuard<Arc<TokioRuntime>> =
      env.get_rust_field(
        interop_state,
        TOKIO_RUNTIME_STATE_FIELD_NAME
      ).unwrap();
    tokio_runtime_arc.clone()
  };

  // TODO: is the extra scope required to free the mutex?
  let state_manager_channel_sender = {
    let state_manager_channel_sender: MutexGuard<Sender<StateManagerRequest>> =
      env.get_rust_field(
        interop_state,
        STATE_MANAGER_CHANNEL_SENDER_STATE_FIELD_NAME
      ).unwrap();

      state_manager_channel_sender.clone()
  };

  (tokio_runtime, state_manager_channel_sender)
}
