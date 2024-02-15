use crate::engine_prelude::*;

use serde::Deserialize;
use serde_with::serde_as;

use super::*;

/// Encoder of SBOR values to schema-aware Programmatic JSON format.
pub struct ProgrammaticJsonEncoder<'a> {
    address_encoder: &'a AddressBech32Encoder,
}

impl<'a> ProgrammaticJsonEncoder<'a> {
    /// Creates an instance which will use the given context (for address encoding).
    pub fn new(mapping_context: &'a MappingContext) -> Self {
        Self {
            address_encoder: &mapping_context.address_encoder,
        }
    }

    /// Encodes the given SBOR bytes to Programmatic JSON (returned as `serde` JSON tree), using the
    /// given schema/type information for human-readable field and type names.
    pub fn encode(
        &self,
        payload_bytes: Vec<u8>,
        schema: &SchemaV1<ScryptoCustomSchema>,
        type_id: LocalTypeId,
    ) -> Result<serde_json::Value, MappingError> {
        let raw_payload = RawScryptoPayload::new_from_valid_owned(payload_bytes);
        let serializable = raw_payload.serializable(SerializationParameters::WithSchema {
            mode: SerializationMode::Programmatic,
            custom_context: ScryptoValueDisplayContext::with_optional_bech32(Some(
                self.address_encoder,
            )),
            schema,
            type_id,
            depth_limit: SCRYPTO_SBOR_V1_MAX_DEPTH,
        });
        serde_json::to_value(serializable).map_err(|_error| MappingError::SubstateValue {
            bytes: raw_payload.payload_bytes().to_vec(),
            message: "cannot render as programmatic json".to_string(),
        })
    }
}

/// Decoder of Programmatic JSON format to SBOR values.
pub struct ProgrammaticJsonDecoder<'a> {
    address_decoder: &'a AddressBech32Decoder,
}

impl<'a> ProgrammaticJsonDecoder<'a> {
    /// Creates an instance which will use the given context (for address decoding).
    pub fn new(extraction_context: &'a ExtractionContext) -> Self {
        Self {
            address_decoder: &extraction_context.address_decoder,
        }
    }

    /// Encodes the Programmatic JSON (given as `serde` JSON tree) to [`ScryptoValue`].
    /// Please note that schema/type information is not required for decoding (unlike
    /// [`ProgrammaticJsonEncoder::encode`]).
    pub fn decode(&self, json_value: serde_json::Value) -> Result<ScryptoValue, ExtractionError> {
        serde_json::from_value::<ProgrammaticScryptoValue>(json_value)
            .map_err(|_error| ExtractionError::InvalidProgrammaticJson {
                message: "while building SBOR struct from serde value".to_string(),
            })?
            .extract_scrypto_value(self.address_decoder)
    }
}

// IMPLEMENTATION NOTE:
//
// At the time of writing this, the Engine did not offer any decoding support for the programmatic
// JSON format. The rest of our implementation below is based on the RET's approach captured at the
// exact linked version:
//
// https://github.com/radixdlt/radix-engine-toolkit/blob/9eb4b6f14bbad89ea24afbd7f17977cae1c6d6ae/radix-engine-toolkit-core/src/functions/scrypto_sbor.rs#L92
// (with majority of logic hidden within serde annotation on `ProgrammaticScryptoValue` struct)
//
// The adjustments made here:
// - Removed the `serde`'s `Serialize` support (we use the Engine's impl for serialization and only
//   want to implement `Deserialize` here).
// - Re-worked error detection (and surfacing) related to "network HRP mismatch" on addresses
//   contained within JSON tree:
//   - Mostly needed to accommodate the Node's usage of `AddressBech32Decoder`.
//   - This was achieved by "parsing" the `NodeId` to untouched string, i.e. deferring the bech32m
//     decoding to another step taking place after `serde`.
// - Added the "roundtrip serialize/deserialize" test (to ensure compatibility with Engine's
//   serialization) and the "network HRP mismatch" test.

#[serde_as]
#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(tag = "kind")]
pub enum ProgrammaticScryptoValue {
    Bool {
        value: bool,
    },
    I8 {
        #[serde_as(as = "serde_with::DisplayFromStr")]
        value: i8,
    },
    I16 {
        #[serde_as(as = "serde_with::DisplayFromStr")]
        value: i16,
    },
    I32 {
        #[serde_as(as = "serde_with::DisplayFromStr")]
        value: i32,
    },
    I64 {
        #[serde_as(as = "serde_with::DisplayFromStr")]
        value: i64,
    },
    I128 {
        #[serde_as(as = "serde_with::DisplayFromStr")]
        value: i128,
    },
    U8 {
        #[serde_as(as = "serde_with::DisplayFromStr")]
        value: u8,
    },
    U16 {
        #[serde_as(as = "serde_with::DisplayFromStr")]
        value: u16,
    },
    U32 {
        #[serde_as(as = "serde_with::DisplayFromStr")]
        value: u32,
    },
    U64 {
        #[serde_as(as = "serde_with::DisplayFromStr")]
        value: u64,
    },
    U128 {
        #[serde_as(as = "serde_with::DisplayFromStr")]
        value: u128,
    },
    String {
        value: String,
    },
    Enum {
        #[serde(rename = "variant_id")]
        #[serde_as(as = "serde_with::DisplayFromStr")]
        discriminator: u8,
        fields: Vec<ProgrammaticScryptoValue>,
    },
    Array {
        #[serde(rename = "element_kind")]
        element_value_kind: ProgrammaticScryptoValueKind,
        elements: Vec<ProgrammaticScryptoValue>,
    },
    Tuple {
        fields: Vec<ProgrammaticScryptoValue>,
    },
    Map {
        #[serde(rename = "key_kind")]
        key_value_kind: ProgrammaticScryptoValueKind,
        #[serde(rename = "value_kind")]
        value_value_kind: ProgrammaticScryptoValueKind,
        #[serde_as(as = "Vec<serde_with::FromInto<MapEntry<ProgrammaticScryptoValue>>>")]
        entries: Vec<(ProgrammaticScryptoValue, ProgrammaticScryptoValue)>,
    },
    Reference {
        value: SerializableNodeId,
    },
    Own {
        value: SerializableNodeId,
    },
    Decimal {
        #[serde_as(as = "serde_with::DisplayFromStr")]
        value: Decimal,
    },
    PreciseDecimal {
        #[serde_as(as = "serde_with::DisplayFromStr")]
        value: PreciseDecimal,
    },
    NonFungibleLocalId {
        #[serde_as(as = "serde_with::DisplayFromStr")]
        value: NonFungibleLocalId,
    },
    Bytes {
        #[serde(rename = "element_kind")]
        element_value_kind: ProgrammaticScryptoValueKind,

        #[serde_as(as = "serde_with::hex::Hex")]
        #[serde(rename = "hex")]
        value: Vec<u8>,
    },
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProgrammaticScryptoValueKind {
    Bool,
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    String,
    Enum,
    Array,
    Tuple,
    Map,
    Reference,
    Own,
    Decimal,
    PreciseDecimal,
    NonFungibleLocalId,
}

impl From<ProgrammaticScryptoValueKind> for ScryptoValueKind {
    fn from(value: ProgrammaticScryptoValueKind) -> Self {
        match value {
            ProgrammaticScryptoValueKind::Bool => Self::Bool,
            ProgrammaticScryptoValueKind::I8 => Self::I8,
            ProgrammaticScryptoValueKind::I16 => Self::I16,
            ProgrammaticScryptoValueKind::I32 => Self::I32,
            ProgrammaticScryptoValueKind::I64 => Self::I64,
            ProgrammaticScryptoValueKind::I128 => Self::I128,
            ProgrammaticScryptoValueKind::U8 => Self::U8,
            ProgrammaticScryptoValueKind::U16 => Self::U16,
            ProgrammaticScryptoValueKind::U32 => Self::U32,
            ProgrammaticScryptoValueKind::U64 => Self::U64,
            ProgrammaticScryptoValueKind::U128 => Self::U128,
            ProgrammaticScryptoValueKind::String => Self::String,
            ProgrammaticScryptoValueKind::Enum => Self::Enum,
            ProgrammaticScryptoValueKind::Array => Self::Array,
            ProgrammaticScryptoValueKind::Tuple => Self::Tuple,
            ProgrammaticScryptoValueKind::Map => Self::Map,
            ProgrammaticScryptoValueKind::Reference => {
                Self::Custom(ScryptoCustomValueKind::Reference)
            }
            ProgrammaticScryptoValueKind::Own => Self::Custom(ScryptoCustomValueKind::Own),
            ProgrammaticScryptoValueKind::Decimal => Self::Custom(ScryptoCustomValueKind::Decimal),
            ProgrammaticScryptoValueKind::PreciseDecimal => {
                Self::Custom(ScryptoCustomValueKind::PreciseDecimal)
            }
            ProgrammaticScryptoValueKind::NonFungibleLocalId => {
                Self::Custom(ScryptoCustomValueKind::NonFungibleLocalId)
            }
        }
    }
}

impl From<ScryptoValueKind> for ProgrammaticScryptoValueKind {
    fn from(value: ScryptoValueKind) -> Self {
        match value {
            ScryptoValueKind::Bool => Self::Bool,
            ScryptoValueKind::I8 => Self::I8,
            ScryptoValueKind::I16 => Self::I16,
            ScryptoValueKind::I32 => Self::I32,
            ScryptoValueKind::I64 => Self::I64,
            ScryptoValueKind::I128 => Self::I128,
            ScryptoValueKind::U8 => Self::U8,
            ScryptoValueKind::U16 => Self::U16,
            ScryptoValueKind::U32 => Self::U32,
            ScryptoValueKind::U64 => Self::U64,
            ScryptoValueKind::U128 => Self::U128,
            ScryptoValueKind::String => Self::String,
            ScryptoValueKind::Enum => Self::Enum,
            ScryptoValueKind::Array => Self::Array,
            ScryptoValueKind::Tuple => Self::Tuple,
            ScryptoValueKind::Map => Self::Map,
            ScryptoValueKind::Custom(ScryptoCustomValueKind::Reference) => Self::Reference,
            ScryptoValueKind::Custom(ScryptoCustomValueKind::Own) => Self::Own,
            ScryptoValueKind::Custom(ScryptoCustomValueKind::Decimal) => Self::Decimal,
            ScryptoValueKind::Custom(ScryptoCustomValueKind::PreciseDecimal) => {
                Self::PreciseDecimal
            }
            ScryptoValueKind::Custom(ScryptoCustomValueKind::NonFungibleLocalId) => {
                Self::NonFungibleLocalId
            }
        }
    }
}

impl ProgrammaticScryptoValue {
    pub fn extract_scrypto_value(
        &self,
        address_decoder: &AddressBech32Decoder,
    ) -> Result<ScryptoValue, ExtractionError> {
        Ok(match self {
            Self::Bool { value } => ScryptoValue::Bool { value: *value },
            Self::I8 { value } => ScryptoValue::I8 { value: *value },
            Self::I16 { value } => ScryptoValue::I16 { value: *value },
            Self::I32 { value } => ScryptoValue::I32 { value: *value },
            Self::I64 { value } => ScryptoValue::I64 { value: *value },
            Self::I128 { value } => ScryptoValue::I128 { value: *value },
            Self::U8 { value } => ScryptoValue::U8 { value: *value },
            Self::U16 { value } => ScryptoValue::U16 { value: *value },
            Self::U32 { value } => ScryptoValue::U32 { value: *value },
            Self::U64 { value } => ScryptoValue::U64 { value: *value },
            Self::U128 { value } => ScryptoValue::U128 { value: *value },
            Self::String { value } => ScryptoValue::String {
                value: value.clone(),
            },
            Self::Enum {
                discriminator,
                fields,
            } => ScryptoValue::Enum {
                discriminator: *discriminator,
                fields: fields
                    .iter()
                    .map(|field| field.extract_scrypto_value(address_decoder))
                    .collect::<Result<Vec<_>, _>>()?,
            },
            Self::Array {
                element_value_kind,
                elements,
            } => ScryptoValue::Array {
                element_value_kind: (*element_value_kind).into(),
                elements: elements
                    .iter()
                    .map(|field| field.extract_scrypto_value(address_decoder))
                    .collect::<Result<Vec<_>, _>>()?,
            },
            Self::Tuple { fields } => ScryptoValue::Tuple {
                fields: fields
                    .iter()
                    .map(|field| field.extract_scrypto_value(address_decoder))
                    .collect::<Result<Vec<_>, _>>()?,
            },
            Self::Map {
                key_value_kind,
                value_value_kind,
                entries,
            } => ScryptoValue::Map {
                key_value_kind: (*key_value_kind).into(),
                value_value_kind: (*value_value_kind).into(),
                entries: entries
                    .iter()
                    .map(|(key, value)| {
                        Ok::<_, ExtractionError>((
                            key.extract_scrypto_value(address_decoder)?,
                            value.extract_scrypto_value(address_decoder)?,
                        ))
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            },
            Self::Reference { value } => ScryptoValue::Custom {
                value: ScryptoCustomValue::Reference(Reference(value.to_node_id(address_decoder)?)),
            },
            Self::Own { value } => ScryptoValue::Custom {
                value: ScryptoCustomValue::Own(Own(value.to_node_id(address_decoder)?)),
            },
            Self::Decimal { value } => ScryptoValue::Custom {
                value: ScryptoCustomValue::Decimal(*value),
            },
            Self::PreciseDecimal { value } => ScryptoValue::Custom {
                value: ScryptoCustomValue::PreciseDecimal(*value),
            },
            Self::NonFungibleLocalId { value } => ScryptoValue::Custom {
                value: ScryptoCustomValue::NonFungibleLocalId(value.clone()),
            },
            Self::Bytes {
                element_value_kind,
                value,
            } => ScryptoValue::Array {
                element_value_kind: (*element_value_kind).into(),
                elements: value
                    .iter()
                    .map(|value| ScryptoValue::U8 { value: *value })
                    .collect(),
            },
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
pub struct MapEntry<T> {
    pub key: T,
    pub value: T,
}

impl<T> From<(T, T)> for MapEntry<T> {
    fn from((key, value): (T, T)) -> Self {
        Self { key, value }
    }
}

impl<T> From<MapEntry<T>> for (T, T) {
    fn from(value: MapEntry<T>) -> Self {
        (value.key, value.value)
    }
}

#[serde_as]
#[derive(Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
pub struct SerializableNodeId {
    pub address: String,
}

impl SerializableNodeId {
    pub fn to_node_id(
        &self,
        address_decoder: &AddressBech32Decoder,
    ) -> Result<NodeId, ExtractionError> {
        address_decoder
            .validate_and_decode(self.address.as_str())
            .ok()
            .and_then(|(_entity_type, bytes)| bytes.try_into().ok())
            .map(NodeId)
            .ok_or(ExtractionError::InvalidAddress)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use radix_engine::blueprints::consensus_manager::{
        ActiveValidatorSet, ConsensusManagerBlueprint,
        ConsensusManagerCurrentValidatorSetFieldPayload, ConsensusManagerCurrentValidatorSetV1,
        Validator, VersionedConsensusManagerCurrentValidatorSet,
    };

    #[test]
    fn decodes_exactly_what_was_encoded() {
        // just some hardcoded knowledge of the struct's schema: (needed by programmatic json format)
        let schema = ConsensusManagerBlueprint::definition()
            .schema
            .schema
            .into_latest();
        let type_id = LocalTypeId::SchemaLocalIndex(12);
        let network = NetworkDefinition::mainnet(); // exact value irrelevant

        // the struct to perform the round-trip with:
        let original_payload = ConsensusManagerCurrentValidatorSetFieldPayload {
            content: VersionedConsensusManagerCurrentValidatorSet::V1(
                ConsensusManagerCurrentValidatorSetV1 {
                    validator_set: ActiveValidatorSet {
                        validators_by_stake_desc: indexmap!(
                            ComponentAddress::new_or_panic(
                                [EntityType::GlobalValidator as u8; 30]
                            ) => Validator {
                                key: Secp256k1PublicKey([7; 33]),
                                stake: dec!("13.37")
                            }
                        ),
                    },
                },
            ),
        };

        // struct -> sbor_bytes -> serde JSON:
        let original_bytes = scrypto_encode(&original_payload).unwrap();
        let json_value = ProgrammaticJsonEncoder::new(&MappingContext::new(&network))
            .encode(original_bytes.clone(), &schema, type_id)
            .unwrap();

        // serde JSON -> sbor_bytes -> struct:
        let retrieved_value = ProgrammaticJsonDecoder::new(&ExtractionContext::new(&network))
            .decode(json_value)
            .unwrap();
        let retrieved_payload: ConsensusManagerCurrentValidatorSetFieldPayload =
            scrypto_decode(&scrypto_encode(&retrieved_value).unwrap()).unwrap();

        // assert the struct survived the round-trip:
        assert_eq!(retrieved_payload, original_payload);
    }

    #[test]
    fn non_matching_network_causes_error_when_decoding_address_within_programmatic_json() {
        // just some hardcoded knowledge of the struct's schema: (needed by programmatic json format)
        let schema = ConsensusManagerBlueprint::definition()
            .schema
            .schema
            .into_latest();
        let type_id = LocalTypeId::SchemaLocalIndex(12);

        // the struct with address:
        let original_payload = ConsensusManagerCurrentValidatorSetFieldPayload {
            content: VersionedConsensusManagerCurrentValidatorSet::V1(
                ConsensusManagerCurrentValidatorSetV1 {
                    validator_set: ActiveValidatorSet {
                        validators_by_stake_desc: indexmap!(
                            ComponentAddress::new_or_panic(
                                [EntityType::GlobalValidator as u8; 30]
                            ) => Validator {
                                key: Secp256k1PublicKey([7; 33]),
                                stake: dec!("13.37")
                            }
                        ),
                    },
                },
            ),
        };

        // serialize and expect error on deserialization:
        let original_bytes = scrypto_encode(&original_payload).unwrap();
        let json_value =
            ProgrammaticJsonEncoder::new(&MappingContext::new(&NetworkDefinition::nebunet()))
                .encode(original_bytes.clone(), &schema, type_id)
                .unwrap();
        let error =
            ProgrammaticJsonDecoder::new(&ExtractionContext::new(&NetworkDefinition::ansharnet()))
                .decode(json_value)
                .err()
                .unwrap();

        // assert the error's type
        assert!(matches!(error, ExtractionError::InvalidAddress));
    }
}
