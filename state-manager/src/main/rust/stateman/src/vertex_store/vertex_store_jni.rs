use jni::JNIEnv;
use jni::objects::{JClass, JObject};
use jni::sys::jbyteArray;
use crate::interop_state::INTEROP_STATE_VERTEX_STORE_REF;
use crate::jni_util::use_rust_jni_obj;
use crate::vertex_store::VertexStore;

#[no_mangle]
extern "system" fn Java_com_radixdlt_vertexstore_RustVertexStore_insertVertex(
    env: JNIEnv,
    _class: JClass,
    interop_state: JObject,
    j_vertex: jbyteArray
) {
    use_vertex_store(env, interop_state, |vertex_store| {
        let vertex: Vec<u8> = env.convert_byte_array(j_vertex)
            .expect("Can't convert vertex byte array to vec");
        vertex_store.insert_vertex(vertex);
    });
}

#[no_mangle]
extern "system" fn Java_com_radixdlt_vertexstore_RustVertexStore_containsVertex(
    env: JNIEnv,
    _class: JClass,
    interop_state: JObject,
    j_vertex: jbyteArray
) -> bool {
    use_vertex_store(env, interop_state, |vertex_store| {
        let vertex: Vec<u8> = env.convert_byte_array(j_vertex)
            .expect("Can't convert vertex byte array to vec");
        vertex_store.contains_vertex(vertex)
    })
}

fn use_vertex_store<F, R>(
    env: JNIEnv,
    interop_state: JObject,
    thunk: F
) -> R where F: FnOnce(&mut VertexStore) -> R {
    use_rust_jni_obj(env, interop_state, INTEROP_STATE_VERTEX_STORE_REF, thunk)
}
