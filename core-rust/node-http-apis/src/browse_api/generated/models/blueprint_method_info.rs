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
pub struct BlueprintMethodInfo {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "receiver")]
    pub receiver: Box<crate::browse_api::generated::models::BlueprintMethodReceiverInfo>,
    #[serde(rename = "input_type_reference")]
    pub input_type_reference: Option<crate::browse_api::generated::models::BlueprintResolvedTypeReference>, // Using Option permits Default trait; Will always be Some in normal use
    #[serde(rename = "output_type_reference")]
    pub output_type_reference: Option<crate::browse_api::generated::models::BlueprintResolvedTypeReference>, // Using Option permits Default trait; Will always be Some in normal use
    #[serde(rename = "authorization")]
    pub authorization: Option<crate::browse_api::generated::models::BlueprintMethodAuthorization>, // Using Option permits Default trait; Will always be Some in normal use
}

impl BlueprintMethodInfo {
    pub fn new(name: String, receiver: crate::browse_api::generated::models::BlueprintMethodReceiverInfo, input_type_reference: crate::browse_api::generated::models::BlueprintResolvedTypeReference, output_type_reference: crate::browse_api::generated::models::BlueprintResolvedTypeReference, authorization: crate::browse_api::generated::models::BlueprintMethodAuthorization) -> BlueprintMethodInfo {
        BlueprintMethodInfo {
            name,
            receiver: Box::new(receiver),
            input_type_reference: Option::Some(input_type_reference),
            output_type_reference: Option::Some(output_type_reference),
            authorization: Option::Some(authorization),
        }
    }
}


