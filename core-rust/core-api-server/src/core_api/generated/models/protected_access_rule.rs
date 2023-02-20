/*
 * Babylon Core API
 *
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node. It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Heavy load may impact the node's function.  If you require queries against historical ledger state, you may also wish to consider using the [Gateway API](https://betanet-gateway.redoc.ly/). 
 *
 * The version of the OpenAPI document: 0.3.0
 * 
 * Generated by: https://openapi-generator.tech
 */




#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct ProtectedAccessRule {
    #[serde(rename = "type")]
    pub _type: crate::core_api::generated::models::AccessRuleType,
    #[serde(rename = "access_rule")]
    pub access_rule: Option<crate::core_api::generated::models::AccessRuleNode>, // Using Option permits Default trait; Will always be Some in normal use
}

impl ProtectedAccessRule {
    pub fn new(_type: crate::core_api::generated::models::AccessRuleType, access_rule: crate::core_api::generated::models::AccessRuleNode) -> ProtectedAccessRule {
        ProtectedAccessRule {
            _type,
            access_rule: Option::Some(access_rule),
        }
    }
}


