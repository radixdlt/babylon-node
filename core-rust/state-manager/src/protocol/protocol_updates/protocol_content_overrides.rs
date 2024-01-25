use super::definitions::*;
use crate::protocol::*;
use radix_engine::types::scrypto_encode;
use sbor::Sbor;

use radix_engine_common::ScryptoSbor;
use utils::prelude::*;

type Overrides<X> = <X as ProtocolUpdateDefinition>::Overrides;

/// Intended to be an easy-to-use type in rust (or java) for configuring each update.
#[derive(Default, ScryptoSbor)]
pub struct ProtocolUpdateContentOverrides {
    anemone: Option<Overrides<AnemoneProtocolUpdateDefinition>>,
    custom: HashMap<ProtocolVersionName, Overrides<CustomProtocolUpdateDefinition>>,
}

impl ProtocolUpdateContentOverrides {
    pub fn empty() -> Self {
        Default::default()
    }

    pub fn with_anemone(mut self, config: Overrides<AnemoneProtocolUpdateDefinition>) -> Self {
        self.anemone = Some(config);
        self
    }

    pub fn with_custom(
        mut self,
        custom_name: ProtocolVersionName,
        config: Overrides<CustomProtocolUpdateDefinition>,
    ) -> Self {
        if !CustomProtocolUpdateDefinition::matches(&custom_name) {
            panic!(
                "Not an allowed custom protocol update name: {}",
                custom_name
            );
        }
        self.custom.insert(custom_name, config);
        self
    }
}

impl From<ProtocolUpdateContentOverrides> for RawProtocolUpdateContentOverrides {
    fn from(value: ProtocolUpdateContentOverrides) -> Self {
        let mut map = HashMap::default();

        if let Some(config) = value.anemone {
            map.insert(
                ProtocolVersionName::of(ANEMONE_PROTOCOL_VERSION).unwrap(),
                scrypto_encode(&config).unwrap(),
            );
        }

        for (update_name, config) in value.custom {
            if CustomProtocolUpdateDefinition::matches(&update_name) {
                map.insert(update_name, scrypto_encode(&config).unwrap());
            }
        }

        Self(map)
    }
}

#[derive(Default, Clone, Debug, Eq, PartialEq, Sbor)]
#[sbor(transparent)]
pub struct RawProtocolUpdateContentOverrides(HashMap<ProtocolVersionName, Vec<u8>>);

impl RawProtocolUpdateContentOverrides {
    pub fn none() -> Self {
        Default::default()
    }

    pub fn iter(&self) -> hash_map::Iter<ProtocolVersionName, Vec<u8>> {
        self.0.iter()
    }

    pub fn get(&self, protocol_version_name: &ProtocolVersionName) -> Option<&[u8]> {
        self.0.get(protocol_version_name).map(|x| x.as_ref())
    }
}
