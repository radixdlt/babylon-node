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
pub struct V0StateEpochResponse {
    /// An integer between `0` and `10^10`, marking the current epoch
    #[serde(rename = "epoch")]
    pub epoch: i64,
}

impl V0StateEpochResponse {
    pub fn new(epoch: i64) -> V0StateEpochResponse {
        V0StateEpochResponse {
            epoch,
        }
    }
}


