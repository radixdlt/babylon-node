/*
 * Babylon Core API
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 0.1.0
 * 
 * Generated by: https://openapi-generator.tech
 */




#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct FungibleResourceAmountAllOf {
    /// The string-encoded decimal subunits of the amount (`10^-18`) in a signed 256-bit integer. This is string-encoded as it doesn't fit well into common numeric types. 
    #[serde(rename = "amount_attos")]
    pub amount_attos: String,
}

impl FungibleResourceAmountAllOf {
    pub fn new(amount_attos: String) -> FungibleResourceAmountAllOf {
        FungibleResourceAmountAllOf {
            amount_attos,
        }
    }
}


