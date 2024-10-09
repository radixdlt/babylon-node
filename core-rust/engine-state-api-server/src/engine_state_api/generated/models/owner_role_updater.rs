/*
 * Engine State API (Beta)
 *
 * **This API is currently in Beta**  This specification may experience breaking changes as part of Babylon Node releases. Such changes will be clearly mentioned in the [babylon-node release notes](https://github.com/radixdlt/babylon-node/releases). We advise against using this API for business-critical integrations before the `version` indicated above becomes stable, which is expected in Q4 of 2024.  This API provides a complete view of the current ledger state, operating at a relatively low level (i.e. returning Entities' data and type information in a generic way, without interpreting specifics of different native or custom components).  It mirrors how the Radix Engine views the ledger state in its \"System\" layer, and thus can be useful for Scrypto developers, who need to inspect how the Engine models and stores their application's state, or how an interface / authentication scheme of another component looks like. 
 *
 * The version of the OpenAPI document: v1.2.3
 * 
 * Generated by: https://openapi-generator.tech
 */

/// OwnerRoleUpdater : The ability to update the Owner: - None: Owner is fixed and cannot be updated by anyone. - Owner: Owner role may only be updated by the owner themselves. - Object: Owner role may be updated by the object containing the access rules. This is currently primarily used for Presecurified objects. 

/// The ability to update the Owner: - None: Owner is fixed and cannot be updated by anyone. - Owner: Owner role may only be updated by the owner themselves. - Object: Owner role may be updated by the object containing the access rules. This is currently primarily used for Presecurified objects. 
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize)]
pub enum OwnerRoleUpdater {
    #[serde(rename = "None")]
    None,
    #[serde(rename = "Owner")]
    Owner,
    #[serde(rename = "Object")]
    Object,

}

impl ToString for OwnerRoleUpdater {
    fn to_string(&self) -> String {
        match self {
            Self::None => String::from("None"),
            Self::Owner => String::from("Owner"),
            Self::Object => String::from("Object"),
        }
    }
}

impl Default for OwnerRoleUpdater {
    fn default() -> OwnerRoleUpdater {
        Self::None
    }
}




