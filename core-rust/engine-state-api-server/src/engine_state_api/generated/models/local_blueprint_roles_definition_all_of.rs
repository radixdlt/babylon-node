/*
 * Engine State API - Babylon (Anemone)
 *
 * This API provides a complete view of the current ledger state, operating at a relatively low level (i.e. returning Entities' data and type information in a generic way, without interpreting specifics of different native or custom components).  It mirrors how the Radix Engine views the ledger state in its \"System\" layer, and thus can be useful for Scrypto developers, who need to inspect how the Engine models and stores their application's state, or how an interface / authentication scheme of another component looks like. 
 *
 * The version of the OpenAPI document: v0.0.1
 * 
 * Generated by: https://openapi-generator.tech
 */

/// LocalBlueprintRolesDefinitionAllOf : Roles are specified by the blueprint.



#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct LocalBlueprintRolesDefinitionAllOf {
    #[serde(rename = "definitions")]
    pub definitions: Vec<crate::engine_state_api::generated::models::BlueprintRoleInfo>,
}

impl LocalBlueprintRolesDefinitionAllOf {
    /// Roles are specified by the blueprint.
    pub fn new(definitions: Vec<crate::engine_state_api::generated::models::BlueprintRoleInfo>) -> LocalBlueprintRolesDefinitionAllOf {
        LocalBlueprintRolesDefinitionAllOf {
            definitions,
        }
    }
}


