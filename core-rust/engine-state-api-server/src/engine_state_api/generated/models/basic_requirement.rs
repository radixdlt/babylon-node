/*
 * Engine State API (Beta)
 *
 * **This API is currently in Beta**  This specification may experience breaking changes as part of Babylon Node releases. Such changes will be clearly mentioned in the [babylon-node release notes](https://github.com/radixdlt/babylon-node/releases). We advise against using this API for business-critical integrations before the `version` indicated above becomes stable, which is expected in Q4 of 2024.  This API provides a complete view of the current ledger state, operating at a relatively low level (i.e. returning Entities' data and type information in a generic way, without interpreting specifics of different native or custom components).  It mirrors how the Radix Engine views the ledger state in its \"System\" layer, and thus can be useful for Scrypto developers, who need to inspect how the Engine models and stores their application's state, or how an interface / authentication scheme of another component looks like. 
 *
 * The version of the OpenAPI document: v1.3.0
 * 
 * Generated by: https://openapi-generator.tech
 */



#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum BasicRequirement {
    #[serde(rename="AllOf")]
    AllOfBasicRequirement {
        #[serde(rename = "list")]
        list: Vec<crate::engine_state_api::generated::models::Requirement>,
    },
    #[serde(rename="AmountOf")]
    AmountOfBasicRequirement {
        #[serde(rename = "amount")]
        amount: String,
        /// A Bech32m-encoded human readable version of a Resource address.
        #[serde(rename = "resource")]
        resource: String,
    },
    #[serde(rename="AnyOf")]
    AnyOfBasicRequirement {
        #[serde(rename = "list")]
        list: Vec<crate::engine_state_api::generated::models::Requirement>,
    },
    #[serde(rename="CountOf")]
    CountOfBasicRequirement {
        #[serde(rename = "count")]
        count: i32,
        #[serde(rename = "list")]
        list: Vec<crate::engine_state_api::generated::models::Requirement>,
    },
    #[serde(rename="Require")]
    RequireBasicRequirement {
        #[serde(rename = "requirement")]
        requirement: Box<crate::engine_state_api::generated::models::Requirement>,
    },
}




