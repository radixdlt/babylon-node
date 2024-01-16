/*
 * Browse API
 *
 * This API provides a complete view of the current ledger state, operating at a relatively low level (i.e. returning Entities' data and type information in a generic way, without interpreting specifics of different native or custom components).  It mirrors how the Radix Engine views the ledger state in its \"System\" layer, and thus can be useful for Scrypto developers, who need to inspect how the Engine models and stores their application's state, or how an interface / authentication scheme of another component looks like. 
 *
 * The version of the OpenAPI document: v0.0.1
 * 
 * Generated by: https://openapi-generator.tech
 */




#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct OwnPackageOnlyBlueprintMethodAuthorization {
    #[serde(rename = "type")]
    pub _type: crate::browse_api::generated::models::BlueprintMethodAuthorizationType,
}

impl OwnPackageOnlyBlueprintMethodAuthorization {
    pub fn new(_type: crate::browse_api::generated::models::BlueprintMethodAuthorizationType) -> OwnPackageOnlyBlueprintMethodAuthorization {
        OwnPackageOnlyBlueprintMethodAuthorization {
            _type,
        }
    }
}


