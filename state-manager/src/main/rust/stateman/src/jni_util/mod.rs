use jni::JNIEnv;
use jni::objects::JObject;

pub fn use_rust_jni_obj<S: Send + 'static, F, R>(
    env: JNIEnv,
    interop_state: JObject,
    state_ref_field_name: &str,
    thunk: F
) -> R where F: FnOnce(&mut S) -> R {
    let mut stored_obj: S = env.take_rust_field(interop_state, state_ref_field_name)
        .expect("Can't take rust object from JNI env");
    let result = thunk(&mut stored_obj);
    env.set_rust_field(interop_state, state_ref_field_name, stored_obj)
        .expect("Can't put rust object into JNI env");
    result
}
