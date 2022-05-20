use jni::JNIEnv;
use jni::objects::JObject;

pub const INTEROP_STATE_PUBLIC_KEY: &str = "publicKey";
pub const INTEROP_STATE_VERTEX_STORE_REF: &str = "vertexStoreRef";
pub const INTEROP_STATE_TRANSACTION_STORE_REF: &str = "transactionStoreRef";

pub struct NodeInfo {
    pub public_key: Vec<u8>
}

pub fn get_node_info(env: JNIEnv, interop_state: JObject) -> NodeInfo {
    let j_public_key = *env.get_field(interop_state, INTEROP_STATE_PUBLIC_KEY, "[B")
        .and_then(|res| res.l())
        .expect("Can't read public key from JNI env");

    let public_key: Vec<u8> = env.convert_byte_array(j_public_key)
        .expect("Can't convert public key byte array to vec");

    NodeInfo { public_key: public_key }
}
