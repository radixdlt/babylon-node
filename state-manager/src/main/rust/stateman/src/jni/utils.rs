use jni::sys::jbyteArray;
use jni::JNIEnv;

use crate::result::{StateManagerError, StateManagerResult, ERRCODE_JNI};

pub fn jni_jbytearray_to_vector(
    env: &JNIEnv,
    jbytearray: jbyteArray,
) -> StateManagerResult<Vec<u8>> {
    env.convert_byte_array(jbytearray)
        .map_err(|jerr| StateManagerError::create(ERRCODE_JNI, jerr.to_string()))
}

pub fn jni_slice_to_jbytearray(env: &JNIEnv, slice: &[u8]) -> jbyteArray {
    // Unwrap looks bad here, but:
    //
    // 1. by looking at the source code of the JNI, it seems this
    // cannot really fail unless OOM.
    //
    // 2. in case this fails, we would still have to map the error
    // code in a jbyteArray, so possibly the only way to solve this is
    // by having a static bytearray to return in this extremely remote
    // case.
    env.byte_array_from_slice(slice).unwrap()
}
