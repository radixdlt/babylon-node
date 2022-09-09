use sbor::Encode;
use scrypto::prelude::scrypto_encode;

pub fn to_hex<T: AsRef<[u8]>>(v: T) -> String {
    hex::encode(v)
}

pub fn to_sbor_hex<T: Encode + ?Sized>(v: &T) -> String {
    to_hex(scrypto_encode(v))
}
