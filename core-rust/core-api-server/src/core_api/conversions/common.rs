use sbor::Encode;
use scrypto::prelude::scrypto_encode;

pub fn to_hex(v: Vec<u8>) -> String {
    hex::encode(v)
}

pub fn to_sbor_hex<T: Encode + ?Sized>(v: &T) -> String {
    to_hex(scrypto_encode(v))
}
