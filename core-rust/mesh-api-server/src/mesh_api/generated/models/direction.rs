/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere. 
 *
 * The version of the OpenAPI document: 1.4.13
 * 
 * Generated by: https://openapi-generator.tech
 */

/// Direction : Used by RelatedTransaction to indicate the direction of the relation (i.e. cross-shard/cross-network sends may reference `backward` to an earlier transaction and async execution may reference `forward`). Can be used to indicate if a transaction relation is from child to parent or the reverse. 

/// Used by RelatedTransaction to indicate the direction of the relation (i.e. cross-shard/cross-network sends may reference `backward` to an earlier transaction and async execution may reference `forward`). Can be used to indicate if a transaction relation is from child to parent or the reverse. 
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize)]
pub enum Direction {
    #[serde(rename = "forward")]
    Forward,
    #[serde(rename = "backward")]
    Backward,

}

impl ToString for Direction {
    fn to_string(&self) -> String {
        match self {
            Self::Forward => String::from("forward"),
            Self::Backward => String::from("backward"),
        }
    }
}

impl Default for Direction {
    fn default() -> Direction {
        Self::Forward
    }
}



