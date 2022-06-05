use crate::result::*;
use sbor::*;

// TODO: simple derive macro?
pub trait JavaStructure: Encode + Decode {
    fn from_java(data: &[u8]) -> StateManagerResult<Self> {
        decode_with_type(data).map_err(|e| {
            StateManagerError::create(ERRCODE_SBOR, format!("SBOR Decode Failed: {:?}", e))
        })
    }

    fn to_java(&self) -> Vec<u8> {
        encode_with_type(self)
    }
}

#[derive(Debug, TypeId, Encode, Decode, PartialEq, Eq, Hash, Clone)]
pub struct Aid {
    pub bytes: Vec<u8>,
}

#[derive(Debug, TypeId, Encode, Decode, PartialEq, Eq, Hash, Clone)]
pub struct Transaction {
    pub payload: Vec<u8>,
    pub id: Aid,
}

impl JavaStructure for Transaction {}

#[cfg(test)]
mod tests {
    use crate::types::*;

    #[derive(Debug, TypeId, Encode, Decode, PartialEq)]
    pub struct TypeA {
        bytes_a: Vec<u8>,
    }

    #[derive(Debug, TypeId, Encode, Decode, PartialEq)]
    pub struct TypeB {
        bytes_b: Vec<u8>,
        a: TypeA,
    }

    impl JavaStructure for TypeB {}

    #[test]
    fn local_sbor_test_transaction() {
        let a0 = TypeA {
            bytes_a: vec![1u8; 32],
        };
        let b0 = TypeB {
            bytes_b: vec![2u8; 64],
            a: a0,
        };
        let sbor0 = b0.to_java();
        let r = TypeB::from_java(&sbor0);
        assert!(r.is_ok());
        let b1 = r.unwrap();
        assert_eq!(b0, b1);
    }
}
