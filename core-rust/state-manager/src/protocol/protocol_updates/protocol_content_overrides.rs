use super::definitions::*;
use crate::engine_prelude::*;
use crate::protocol::*;

type Overrides<X> = <X as ProtocolUpdateDefinition>::Overrides;

/// Intended to be an easy-to-use type in rust (or java) for configuring each update.
#[derive(Default, ScryptoSbor)]
pub struct ProtocolUpdateContentOverrides {
    anemone: Option<Overrides<AnemoneProtocolUpdateDefinition>>,
    bottlenose: Option<Overrides<BottlenoseProtocolUpdateDefinition>>,
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

    pub fn with_bottlenose(
        mut self,
        config: Overrides<BottlenoseProtocolUpdateDefinition>,
    ) -> Self {
        self.bottlenose = Some(config);
        self
    }

    pub fn with_custom(
        mut self,
        custom_name: ProtocolVersionName,
        overrides: Overrides<CustomProtocolUpdateDefinition>,
    ) -> Self {
        if !CustomProtocolUpdateDefinition::matches(custom_name.as_str()) {
            panic!(
                "Not an allowed custom protocol update name: {}",
                custom_name
            );
        }
        self.custom.insert(custom_name, overrides);
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
        if let Some(config) = value.bottlenose {
            map.insert(
                ProtocolVersionName::of(BOTTLENOSE_PROTOCOL_VERSION).unwrap(),
                scrypto_encode(&config).unwrap(),
            );
        }

        for (update_name, config) in value.custom {
            if CustomProtocolUpdateDefinition::matches(update_name.as_str()) {
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
