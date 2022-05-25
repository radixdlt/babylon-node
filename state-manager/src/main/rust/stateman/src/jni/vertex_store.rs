use crate::jni::state_manager::jni_state_manager;
use jni::objects::{JClass, JObject};
use jni::sys::jbyteArray;
use jni::JNIEnv;

#[no_mangle]
extern "system" fn Java_com_radixdlt_vertexstore_RustVertexStore_insertVertex(
    env: JNIEnv,
    _class: JClass,
    interop_state: JObject,
    j_vertex: jbyteArray,
) {
    let state_manager = jni_state_manager(env, interop_state);

    let vertex: Vec<u8> = env.convert_byte_array(j_vertex).unwrap();

    // only get the lock for vertex store
    state_manager
        .vertex_store
        .lock()
        .unwrap()
        .insert_vertex(vertex);
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_vertexstore_RustVertexStore_containsVertex(
    env: JNIEnv,
    _class: JClass,
    interop_state: JObject,
    j_vertex: jbyteArray,
) -> bool {
    let state_manager = jni_state_manager(env, interop_state);

    let vertex: Vec<u8> = env.convert_byte_array(j_vertex).unwrap();

    // only get the lock for vertex store
    let res = state_manager
        .vertex_store
        .lock()
        .unwrap()
        .contains_vertex(vertex);

    res
}
