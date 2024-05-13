/* tslint:disable */
/* eslint-disable */
/**
 * Radix Core API - Babylon (Bottlenose)
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node\'s function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node\'s current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.2.0
 * 
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */


/**
 * 
 * @export
 */
export const SubstateType = {
    BootLoaderModuleFieldVmBoot: 'BootLoaderModuleFieldVmBoot',
    TypeInfoModuleFieldTypeInfo: 'TypeInfoModuleFieldTypeInfo',
    RoleAssignmentModuleFieldOwnerRole: 'RoleAssignmentModuleFieldOwnerRole',
    RoleAssignmentModuleRuleEntry: 'RoleAssignmentModuleRuleEntry',
    RoleAssignmentModuleMutabilityEntry: 'RoleAssignmentModuleMutabilityEntry',
    RoyaltyModuleFieldState: 'RoyaltyModuleFieldState',
    RoyaltyModuleMethodRoyaltyEntry: 'RoyaltyModuleMethodRoyaltyEntry',
    MetadataModuleEntry: 'MetadataModuleEntry',
    PackageFieldRoyaltyAccumulator: 'PackageFieldRoyaltyAccumulator',
    PackageCodeVmTypeEntry: 'PackageCodeVmTypeEntry',
    PackageCodeOriginalCodeEntry: 'PackageCodeOriginalCodeEntry',
    PackageCodeInstrumentedCodeEntry: 'PackageCodeInstrumentedCodeEntry',
    SchemaEntry: 'SchemaEntry',
    PackageBlueprintDefinitionEntry: 'PackageBlueprintDefinitionEntry',
    PackageBlueprintDependenciesEntry: 'PackageBlueprintDependenciesEntry',
    PackageBlueprintRoyaltyEntry: 'PackageBlueprintRoyaltyEntry',
    PackageBlueprintAuthTemplateEntry: 'PackageBlueprintAuthTemplateEntry',
    PackageFieldFunctionAccessRules: 'PackageFieldFunctionAccessRules',
    FungibleResourceManagerFieldDivisibility: 'FungibleResourceManagerFieldDivisibility',
    FungibleResourceManagerFieldTotalSupply: 'FungibleResourceManagerFieldTotalSupply',
    NonFungibleResourceManagerFieldIdType: 'NonFungibleResourceManagerFieldIdType',
    NonFungibleResourceManagerFieldTotalSupply: 'NonFungibleResourceManagerFieldTotalSupply',
    NonFungibleResourceManagerFieldMutableFields: 'NonFungibleResourceManagerFieldMutableFields',
    NonFungibleResourceManagerDataEntry: 'NonFungibleResourceManagerDataEntry',
    FungibleVaultFieldBalance: 'FungibleVaultFieldBalance',
    FungibleVaultFieldFrozenStatus: 'FungibleVaultFieldFrozenStatus',
    NonFungibleVaultFieldBalance: 'NonFungibleVaultFieldBalance',
    NonFungibleVaultFieldFrozenStatus: 'NonFungibleVaultFieldFrozenStatus',
    NonFungibleVaultContentsIndexEntry: 'NonFungibleVaultContentsIndexEntry',
    ConsensusManagerFieldConfig: 'ConsensusManagerFieldConfig',
    ConsensusManagerFieldState: 'ConsensusManagerFieldState',
    ConsensusManagerFieldCurrentValidatorSet: 'ConsensusManagerFieldCurrentValidatorSet',
    ConsensusManagerFieldCurrentProposalStatistic: 'ConsensusManagerFieldCurrentProposalStatistic',
    ConsensusManagerFieldCurrentTimeRoundedToMinutes: 'ConsensusManagerFieldCurrentTimeRoundedToMinutes',
    ConsensusManagerFieldCurrentTime: 'ConsensusManagerFieldCurrentTime',
    ConsensusManagerFieldValidatorRewards: 'ConsensusManagerFieldValidatorRewards',
    ConsensusManagerRegisteredValidatorsByStakeIndexEntry: 'ConsensusManagerRegisteredValidatorsByStakeIndexEntry',
    ValidatorFieldState: 'ValidatorFieldState',
    ValidatorFieldProtocolUpdateReadinessSignal: 'ValidatorFieldProtocolUpdateReadinessSignal',
    AccountFieldState: 'AccountFieldState',
    AccountVaultEntry: 'AccountVaultEntry',
    AccountResourcePreferenceEntry: 'AccountResourcePreferenceEntry',
    AccountAuthorizedDepositorEntry: 'AccountAuthorizedDepositorEntry',
    AccessControllerFieldState: 'AccessControllerFieldState',
    GenericScryptoComponentFieldState: 'GenericScryptoComponentFieldState',
    GenericKeyValueStoreEntry: 'GenericKeyValueStoreEntry',
    OneResourcePoolFieldState: 'OneResourcePoolFieldState',
    TwoResourcePoolFieldState: 'TwoResourcePoolFieldState',
    MultiResourcePoolFieldState: 'MultiResourcePoolFieldState',
    TransactionTrackerFieldState: 'TransactionTrackerFieldState',
    TransactionTrackerCollectionEntry: 'TransactionTrackerCollectionEntry',
    AccountLockerAccountClaimsEntry: 'AccountLockerAccountClaimsEntry',
    BootLoaderModuleFieldSystemBoot: 'BootLoaderModuleFieldSystemBoot',
    BootLoaderModuleFieldKernelBoot: 'BootLoaderModuleFieldKernelBoot'
} as const;
export type SubstateType = typeof SubstateType[keyof typeof SubstateType];


export function SubstateTypeFromJSON(json: any): SubstateType {
    return SubstateTypeFromJSONTyped(json, false);
}

export function SubstateTypeFromJSONTyped(json: any, ignoreDiscriminator: boolean): SubstateType {
    return json as SubstateType;
}

export function SubstateTypeToJSON(value?: SubstateType | null): any {
    return value as any;
}

