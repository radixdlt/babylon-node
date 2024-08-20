/*
 * Engine State API (Beta)
 *
 * **This API is currently in Beta**  This specification may experience breaking changes as part of Babylon Node releases. Such changes will be clearly mentioned in the [babylon-node release notes](https://github.com/radixdlt/babylon-node/releases). We advise against using this API for business-critical integrations before the `version` indicated above becomes stable, which is expected in Q4 of 2024.  This API provides a complete view of the current ledger state, operating at a relatively low level (i.e. returning Entities' data and type information in a generic way, without interpreting specifics of different native or custom components).  It mirrors how the Radix Engine views the ledger state in its \"System\" layer, and thus can be useful for Scrypto developers, who need to inspect how the Engine models and stores their application's state, or how an interface / authentication scheme of another component looks like. 
 *
 * The version of the OpenAPI document: v1.2.2
 * 
 * Generated by: https://openapi-generator.tech
 */

/// SchemaDefinedTypeReference : Reference to a fully-resolved type within a specific schema.



#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct SchemaDefinedTypeReference {
    #[serde(rename = "type")]
    pub _type: crate::engine_state_api::generated::models::ResolvedTypeReferenceType,
    /// A human-readable name, derived on a best-effort basis from the type info/blueprint/schema. May be missing either because the subject deliberately has no defined name (e.g. in case of an unnamed tuple) or because the name resolution was not successful (e.g. when certain naming conventions are not observed within the relevant definitions). 
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "schema_reference")]
    pub schema_reference: Box<crate::engine_state_api::generated::models::SchemaReference>,
    /// The type's index within the referenced schema.
    #[serde(rename = "index")]
    pub index: i64,
}

impl SchemaDefinedTypeReference {
    /// Reference to a fully-resolved type within a specific schema.
    pub fn new(_type: crate::engine_state_api::generated::models::ResolvedTypeReferenceType, schema_reference: crate::engine_state_api::generated::models::SchemaReference, index: i64) -> SchemaDefinedTypeReference {
        SchemaDefinedTypeReference {
            _type,
            name: None,
            schema_reference: Box::new(schema_reference),
            index,
        }
    }
}


