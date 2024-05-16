/*
 * Engine State API - Babylon (Anemone)
 *
 * **This API is currently in Beta**  This specification may experience breaking changes as part of Babylon Node releases. Such changes will be clearly mentioned in the [babylon-node release notes](https://github.com/radixdlt/babylon-node/releases). We advise against using this API for business-critical integrations before the `version` indicated above becomes stable, which is expected in Q4 of 2024.  This API provides a complete view of the current ledger state, operating at a relatively low level (i.e. returning Entities' data and type information in a generic way, without interpreting specifics of different native or custom components).  It mirrors how the Radix Engine views the ledger state in its \"System\" layer, and thus can be useful for Scrypto developers, who need to inspect how the Engine models and stores their application's state, or how an interface / authentication scheme of another component looks like. 
 *
 * The version of the OpenAPI document: v0.1-beta
 * 
 * Generated by: https://openapi-generator.tech
 */

/// BlueprintResolvedTypeReference : A type, as declared by a blueprint - which means that it may either be a static, fully-resolved type reference, or one of the blueprint's generic type parameters. 


#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum BlueprintResolvedTypeReference {
    #[serde(rename="Generic")]
    BlueprintGenericTypeReference {
        /// An index within `DetailedBlueprintInfo.generic_type_parameters`.
        #[serde(rename = "generic_type_parameter_index")]
        generic_type_parameter_index: i32,
    },
    #[serde(rename="Static")]
    BlueprintStaticTypeReference {
        #[serde(rename = "static_type_reference")]
        static_type_reference: Box<crate::engine_state_api::generated::models::ResolvedTypeReference>,
    },
}




