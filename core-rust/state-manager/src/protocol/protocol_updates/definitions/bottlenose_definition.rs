use crate::engine_prelude::*;
use crate::protocol::protocol_updates::definitions::ScryptoEntriesBatchGenerator;
use crate::protocol::*;

const BOTTLENOSE_ENTRIES: [(&str, ProtocolUpdateEntry); 5] = [
    (
        "bottlenose-owner-role-getter",
        ProtocolUpdateEntry::OwnerRoleGetter,
    ),
    (
        "bottlenose-system-patches",
        ProtocolUpdateEntry::SystemPatches,
    ),
    (
        "bottlenose-locker-package",
        ProtocolUpdateEntry::LockerPackage,
    ),
    (
        "bottlenose-account-try-deposit-or-refund",
        ProtocolUpdateEntry::AccountTryDepositOrRefundBehaviorChanges,
    ),
    (
        "bottlenose-protocol-params-to-state",
        ProtocolUpdateEntry::ProtocolParamsToState,
    ),
];

pub struct BottlenoseProtocolUpdateDefinition;

impl ProtocolUpdateDefinition for BottlenoseProtocolUpdateDefinition {
    type Overrides = ();

    fn create_updater(
        new_protocol_version: &ProtocolVersionName,
        network_definition: &NetworkDefinition,
        _config: Option<Self::Overrides>,
    ) -> Box<dyn ProtocolUpdater> {
        Box::new(BatchedUpdater::new(
            new_protocol_version.clone(),
            Self::state_computer_config(network_definition),
            ScryptoEntriesBatchGenerator::new(network_definition, &BOTTLENOSE_ENTRIES),
        ))
    }
}
