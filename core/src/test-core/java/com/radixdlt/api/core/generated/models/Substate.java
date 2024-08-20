/*
 * Radix Core API
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.2.2
 * 
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */


package com.radixdlt.api.core.generated.models;

import java.util.Objects;
import java.util.Arrays;
import java.util.Map;
import java.util.HashMap;
import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonSubTypes;
import com.fasterxml.jackson.annotation.JsonTypeInfo;
import com.fasterxml.jackson.annotation.JsonTypeName;
import com.fasterxml.jackson.annotation.JsonValue;
import com.radixdlt.api.core.generated.models.AccessControllerFieldStateSubstate;
import com.radixdlt.api.core.generated.models.AccountAuthorizedDepositorEntrySubstate;
import com.radixdlt.api.core.generated.models.AccountFieldStateSubstate;
import com.radixdlt.api.core.generated.models.AccountLockerAccountClaimsEntrySubstate;
import com.radixdlt.api.core.generated.models.AccountResourcePreferenceEntrySubstate;
import com.radixdlt.api.core.generated.models.AccountVaultEntrySubstate;
import com.radixdlt.api.core.generated.models.BootLoaderModuleFieldKernelBootSubstate;
import com.radixdlt.api.core.generated.models.BootLoaderModuleFieldSystemBootSubstate;
import com.radixdlt.api.core.generated.models.BootLoaderModuleFieldVmBootSubstate;
import com.radixdlt.api.core.generated.models.ConsensusManagerFieldConfigSubstate;
import com.radixdlt.api.core.generated.models.ConsensusManagerFieldCurrentProposalStatisticSubstate;
import com.radixdlt.api.core.generated.models.ConsensusManagerFieldCurrentTimeRoundedToMinutesSubstate;
import com.radixdlt.api.core.generated.models.ConsensusManagerFieldCurrentTimeSubstate;
import com.radixdlt.api.core.generated.models.ConsensusManagerFieldCurrentValidatorSetSubstate;
import com.radixdlt.api.core.generated.models.ConsensusManagerFieldStateSubstate;
import com.radixdlt.api.core.generated.models.ConsensusManagerFieldValidatorRewardsSubstate;
import com.radixdlt.api.core.generated.models.ConsensusManagerRegisteredValidatorsByStakeIndexEntrySubstate;
import com.radixdlt.api.core.generated.models.FungibleResourceManagerFieldDivisibilitySubstate;
import com.radixdlt.api.core.generated.models.FungibleResourceManagerFieldTotalSupplySubstate;
import com.radixdlt.api.core.generated.models.FungibleVaultFieldBalanceSubstate;
import com.radixdlt.api.core.generated.models.FungibleVaultFieldFrozenStatusSubstate;
import com.radixdlt.api.core.generated.models.GenericKeyValueStoreEntrySubstate;
import com.radixdlt.api.core.generated.models.GenericScryptoComponentFieldStateSubstate;
import com.radixdlt.api.core.generated.models.MetadataModuleEntrySubstate;
import com.radixdlt.api.core.generated.models.MultiResourcePoolFieldStateSubstate;
import com.radixdlt.api.core.generated.models.NonFungibleResourceManagerDataEntrySubstate;
import com.radixdlt.api.core.generated.models.NonFungibleResourceManagerFieldIdTypeSubstate;
import com.radixdlt.api.core.generated.models.NonFungibleResourceManagerFieldMutableFieldsSubstate;
import com.radixdlt.api.core.generated.models.NonFungibleResourceManagerFieldTotalSupplySubstate;
import com.radixdlt.api.core.generated.models.NonFungibleVaultContentsIndexEntrySubstate;
import com.radixdlt.api.core.generated.models.NonFungibleVaultFieldBalanceSubstate;
import com.radixdlt.api.core.generated.models.NonFungibleVaultFieldFrozenStatusSubstate;
import com.radixdlt.api.core.generated.models.OneResourcePoolFieldStateSubstate;
import com.radixdlt.api.core.generated.models.PackageBlueprintAuthTemplateEntrySubstate;
import com.radixdlt.api.core.generated.models.PackageBlueprintDefinitionEntrySubstate;
import com.radixdlt.api.core.generated.models.PackageBlueprintDependenciesEntrySubstate;
import com.radixdlt.api.core.generated.models.PackageBlueprintRoyaltyEntrySubstate;
import com.radixdlt.api.core.generated.models.PackageCodeInstrumentedCodeEntrySubstate;
import com.radixdlt.api.core.generated.models.PackageCodeOriginalCodeEntrySubstate;
import com.radixdlt.api.core.generated.models.PackageCodeVmTypeEntrySubstate;
import com.radixdlt.api.core.generated.models.PackageFieldRoyaltyAccumulatorSubstate;
import com.radixdlt.api.core.generated.models.RoleAssignmentModuleFieldOwnerRoleSubstate;
import com.radixdlt.api.core.generated.models.RoleAssignmentModuleRuleEntrySubstate;
import com.radixdlt.api.core.generated.models.RoyaltyModuleFieldStateSubstate;
import com.radixdlt.api.core.generated.models.RoyaltyModuleMethodRoyaltyEntrySubstate;
import com.radixdlt.api.core.generated.models.SchemaEntrySubstate;
import com.radixdlt.api.core.generated.models.SubstateType;
import com.radixdlt.api.core.generated.models.TransactionTrackerCollectionEntrySubstate;
import com.radixdlt.api.core.generated.models.TransactionTrackerFieldStateSubstate;
import com.radixdlt.api.core.generated.models.TwoResourcePoolFieldStateSubstate;
import com.radixdlt.api.core.generated.models.TypeInfoModuleFieldTypeInfoSubstate;
import com.radixdlt.api.core.generated.models.ValidatorFieldProtocolUpdateReadinessSignalSubstate;
import com.radixdlt.api.core.generated.models.ValidatorFieldStateSubstate;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


import com.radixdlt.api.core.generated.client.JSON;
/**
 * Substate
 */
@JsonPropertyOrder({
  Substate.JSON_PROPERTY_SUBSTATE_TYPE,
  Substate.JSON_PROPERTY_IS_LOCKED
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
@JsonIgnoreProperties(
  value = "substate_type", // ignore manually set substate_type, it will be automatically generated by Jackson during serialization
  allowSetters = true // allows the substate_type to be set during deserialization
)
@JsonTypeInfo(use = JsonTypeInfo.Id.NAME, include = JsonTypeInfo.As.PROPERTY, property = "substate_type", visible = true)
@JsonSubTypes({
  @JsonSubTypes.Type(value = AccessControllerFieldStateSubstate.class, name = "AccessControllerFieldState"),
  @JsonSubTypes.Type(value = AccessControllerFieldStateSubstate.class, name = "AccessControllerFieldStateSubstate"),
  @JsonSubTypes.Type(value = AccountAuthorizedDepositorEntrySubstate.class, name = "AccountAuthorizedDepositorEntry"),
  @JsonSubTypes.Type(value = AccountAuthorizedDepositorEntrySubstate.class, name = "AccountAuthorizedDepositorEntrySubstate"),
  @JsonSubTypes.Type(value = AccountFieldStateSubstate.class, name = "AccountFieldState"),
  @JsonSubTypes.Type(value = AccountFieldStateSubstate.class, name = "AccountFieldStateSubstate"),
  @JsonSubTypes.Type(value = AccountLockerAccountClaimsEntrySubstate.class, name = "AccountLockerAccountClaimsEntry"),
  @JsonSubTypes.Type(value = AccountLockerAccountClaimsEntrySubstate.class, name = "AccountLockerAccountClaimsEntrySubstate"),
  @JsonSubTypes.Type(value = AccountResourcePreferenceEntrySubstate.class, name = "AccountResourcePreferenceEntry"),
  @JsonSubTypes.Type(value = AccountResourcePreferenceEntrySubstate.class, name = "AccountResourcePreferenceEntrySubstate"),
  @JsonSubTypes.Type(value = AccountVaultEntrySubstate.class, name = "AccountVaultEntry"),
  @JsonSubTypes.Type(value = AccountVaultEntrySubstate.class, name = "AccountVaultEntrySubstate"),
  @JsonSubTypes.Type(value = BootLoaderModuleFieldKernelBootSubstate.class, name = "BootLoaderModuleFieldKernelBoot"),
  @JsonSubTypes.Type(value = BootLoaderModuleFieldKernelBootSubstate.class, name = "BootLoaderModuleFieldKernelBootSubstate"),
  @JsonSubTypes.Type(value = BootLoaderModuleFieldSystemBootSubstate.class, name = "BootLoaderModuleFieldSystemBoot"),
  @JsonSubTypes.Type(value = BootLoaderModuleFieldSystemBootSubstate.class, name = "BootLoaderModuleFieldSystemBootSubstate"),
  @JsonSubTypes.Type(value = BootLoaderModuleFieldVmBootSubstate.class, name = "BootLoaderModuleFieldVmBoot"),
  @JsonSubTypes.Type(value = BootLoaderModuleFieldVmBootSubstate.class, name = "BootLoaderModuleFieldVmBootSubstate"),
  @JsonSubTypes.Type(value = ConsensusManagerFieldConfigSubstate.class, name = "ConsensusManagerFieldConfig"),
  @JsonSubTypes.Type(value = ConsensusManagerFieldConfigSubstate.class, name = "ConsensusManagerFieldConfigSubstate"),
  @JsonSubTypes.Type(value = ConsensusManagerFieldCurrentProposalStatisticSubstate.class, name = "ConsensusManagerFieldCurrentProposalStatistic"),
  @JsonSubTypes.Type(value = ConsensusManagerFieldCurrentProposalStatisticSubstate.class, name = "ConsensusManagerFieldCurrentProposalStatisticSubstate"),
  @JsonSubTypes.Type(value = ConsensusManagerFieldCurrentTimeSubstate.class, name = "ConsensusManagerFieldCurrentTime"),
  @JsonSubTypes.Type(value = ConsensusManagerFieldCurrentTimeRoundedToMinutesSubstate.class, name = "ConsensusManagerFieldCurrentTimeRoundedToMinutes"),
  @JsonSubTypes.Type(value = ConsensusManagerFieldCurrentTimeRoundedToMinutesSubstate.class, name = "ConsensusManagerFieldCurrentTimeRoundedToMinutesSubstate"),
  @JsonSubTypes.Type(value = ConsensusManagerFieldCurrentTimeSubstate.class, name = "ConsensusManagerFieldCurrentTimeSubstate"),
  @JsonSubTypes.Type(value = ConsensusManagerFieldCurrentValidatorSetSubstate.class, name = "ConsensusManagerFieldCurrentValidatorSet"),
  @JsonSubTypes.Type(value = ConsensusManagerFieldCurrentValidatorSetSubstate.class, name = "ConsensusManagerFieldCurrentValidatorSetSubstate"),
  @JsonSubTypes.Type(value = ConsensusManagerFieldStateSubstate.class, name = "ConsensusManagerFieldState"),
  @JsonSubTypes.Type(value = ConsensusManagerFieldStateSubstate.class, name = "ConsensusManagerFieldStateSubstate"),
  @JsonSubTypes.Type(value = ConsensusManagerFieldValidatorRewardsSubstate.class, name = "ConsensusManagerFieldValidatorRewards"),
  @JsonSubTypes.Type(value = ConsensusManagerFieldValidatorRewardsSubstate.class, name = "ConsensusManagerFieldValidatorRewardsSubstate"),
  @JsonSubTypes.Type(value = ConsensusManagerRegisteredValidatorsByStakeIndexEntrySubstate.class, name = "ConsensusManagerRegisteredValidatorsByStakeIndexEntry"),
  @JsonSubTypes.Type(value = ConsensusManagerRegisteredValidatorsByStakeIndexEntrySubstate.class, name = "ConsensusManagerRegisteredValidatorsByStakeIndexEntrySubstate"),
  @JsonSubTypes.Type(value = FungibleResourceManagerFieldDivisibilitySubstate.class, name = "FungibleResourceManagerFieldDivisibility"),
  @JsonSubTypes.Type(value = FungibleResourceManagerFieldDivisibilitySubstate.class, name = "FungibleResourceManagerFieldDivisibilitySubstate"),
  @JsonSubTypes.Type(value = FungibleResourceManagerFieldTotalSupplySubstate.class, name = "FungibleResourceManagerFieldTotalSupply"),
  @JsonSubTypes.Type(value = FungibleResourceManagerFieldTotalSupplySubstate.class, name = "FungibleResourceManagerFieldTotalSupplySubstate"),
  @JsonSubTypes.Type(value = FungibleVaultFieldBalanceSubstate.class, name = "FungibleVaultFieldBalance"),
  @JsonSubTypes.Type(value = FungibleVaultFieldBalanceSubstate.class, name = "FungibleVaultFieldBalanceSubstate"),
  @JsonSubTypes.Type(value = FungibleVaultFieldFrozenStatusSubstate.class, name = "FungibleVaultFieldFrozenStatus"),
  @JsonSubTypes.Type(value = FungibleVaultFieldFrozenStatusSubstate.class, name = "FungibleVaultFieldFrozenStatusSubstate"),
  @JsonSubTypes.Type(value = GenericKeyValueStoreEntrySubstate.class, name = "GenericKeyValueStoreEntry"),
  @JsonSubTypes.Type(value = GenericKeyValueStoreEntrySubstate.class, name = "GenericKeyValueStoreEntrySubstate"),
  @JsonSubTypes.Type(value = GenericScryptoComponentFieldStateSubstate.class, name = "GenericScryptoComponentFieldState"),
  @JsonSubTypes.Type(value = GenericScryptoComponentFieldStateSubstate.class, name = "GenericScryptoComponentFieldStateSubstate"),
  @JsonSubTypes.Type(value = MetadataModuleEntrySubstate.class, name = "MetadataModuleEntry"),
  @JsonSubTypes.Type(value = MetadataModuleEntrySubstate.class, name = "MetadataModuleEntrySubstate"),
  @JsonSubTypes.Type(value = MultiResourcePoolFieldStateSubstate.class, name = "MultiResourcePoolFieldState"),
  @JsonSubTypes.Type(value = MultiResourcePoolFieldStateSubstate.class, name = "MultiResourcePoolFieldStateSubstate"),
  @JsonSubTypes.Type(value = NonFungibleResourceManagerDataEntrySubstate.class, name = "NonFungibleResourceManagerDataEntry"),
  @JsonSubTypes.Type(value = NonFungibleResourceManagerDataEntrySubstate.class, name = "NonFungibleResourceManagerDataEntrySubstate"),
  @JsonSubTypes.Type(value = NonFungibleResourceManagerFieldIdTypeSubstate.class, name = "NonFungibleResourceManagerFieldIdType"),
  @JsonSubTypes.Type(value = NonFungibleResourceManagerFieldIdTypeSubstate.class, name = "NonFungibleResourceManagerFieldIdTypeSubstate"),
  @JsonSubTypes.Type(value = NonFungibleResourceManagerFieldMutableFieldsSubstate.class, name = "NonFungibleResourceManagerFieldMutableFields"),
  @JsonSubTypes.Type(value = NonFungibleResourceManagerFieldMutableFieldsSubstate.class, name = "NonFungibleResourceManagerFieldMutableFieldsSubstate"),
  @JsonSubTypes.Type(value = NonFungibleResourceManagerFieldTotalSupplySubstate.class, name = "NonFungibleResourceManagerFieldTotalSupply"),
  @JsonSubTypes.Type(value = NonFungibleResourceManagerFieldTotalSupplySubstate.class, name = "NonFungibleResourceManagerFieldTotalSupplySubstate"),
  @JsonSubTypes.Type(value = NonFungibleVaultContentsIndexEntrySubstate.class, name = "NonFungibleVaultContentsIndexEntry"),
  @JsonSubTypes.Type(value = NonFungibleVaultContentsIndexEntrySubstate.class, name = "NonFungibleVaultContentsIndexEntrySubstate"),
  @JsonSubTypes.Type(value = NonFungibleVaultFieldBalanceSubstate.class, name = "NonFungibleVaultFieldBalance"),
  @JsonSubTypes.Type(value = NonFungibleVaultFieldBalanceSubstate.class, name = "NonFungibleVaultFieldBalanceSubstate"),
  @JsonSubTypes.Type(value = NonFungibleVaultFieldFrozenStatusSubstate.class, name = "NonFungibleVaultFieldFrozenStatus"),
  @JsonSubTypes.Type(value = NonFungibleVaultFieldFrozenStatusSubstate.class, name = "NonFungibleVaultFieldFrozenStatusSubstate"),
  @JsonSubTypes.Type(value = OneResourcePoolFieldStateSubstate.class, name = "OneResourcePoolFieldState"),
  @JsonSubTypes.Type(value = OneResourcePoolFieldStateSubstate.class, name = "OneResourcePoolFieldStateSubstate"),
  @JsonSubTypes.Type(value = PackageBlueprintAuthTemplateEntrySubstate.class, name = "PackageBlueprintAuthTemplateEntry"),
  @JsonSubTypes.Type(value = PackageBlueprintAuthTemplateEntrySubstate.class, name = "PackageBlueprintAuthTemplateEntrySubstate"),
  @JsonSubTypes.Type(value = PackageBlueprintDefinitionEntrySubstate.class, name = "PackageBlueprintDefinitionEntry"),
  @JsonSubTypes.Type(value = PackageBlueprintDefinitionEntrySubstate.class, name = "PackageBlueprintDefinitionEntrySubstate"),
  @JsonSubTypes.Type(value = PackageBlueprintDependenciesEntrySubstate.class, name = "PackageBlueprintDependenciesEntry"),
  @JsonSubTypes.Type(value = PackageBlueprintDependenciesEntrySubstate.class, name = "PackageBlueprintDependenciesEntrySubstate"),
  @JsonSubTypes.Type(value = PackageBlueprintRoyaltyEntrySubstate.class, name = "PackageBlueprintRoyaltyEntry"),
  @JsonSubTypes.Type(value = PackageBlueprintRoyaltyEntrySubstate.class, name = "PackageBlueprintRoyaltyEntrySubstate"),
  @JsonSubTypes.Type(value = PackageCodeInstrumentedCodeEntrySubstate.class, name = "PackageCodeInstrumentedCodeEntry"),
  @JsonSubTypes.Type(value = PackageCodeInstrumentedCodeEntrySubstate.class, name = "PackageCodeInstrumentedCodeEntrySubstate"),
  @JsonSubTypes.Type(value = PackageCodeOriginalCodeEntrySubstate.class, name = "PackageCodeOriginalCodeEntry"),
  @JsonSubTypes.Type(value = PackageCodeOriginalCodeEntrySubstate.class, name = "PackageCodeOriginalCodeEntrySubstate"),
  @JsonSubTypes.Type(value = PackageCodeVmTypeEntrySubstate.class, name = "PackageCodeVmTypeEntry"),
  @JsonSubTypes.Type(value = PackageCodeVmTypeEntrySubstate.class, name = "PackageCodeVmTypeEntrySubstate"),
  @JsonSubTypes.Type(value = PackageFieldRoyaltyAccumulatorSubstate.class, name = "PackageFieldRoyaltyAccumulator"),
  @JsonSubTypes.Type(value = PackageFieldRoyaltyAccumulatorSubstate.class, name = "PackageFieldRoyaltyAccumulatorSubstate"),
  @JsonSubTypes.Type(value = RoleAssignmentModuleFieldOwnerRoleSubstate.class, name = "RoleAssignmentModuleFieldOwnerRole"),
  @JsonSubTypes.Type(value = RoleAssignmentModuleFieldOwnerRoleSubstate.class, name = "RoleAssignmentModuleFieldOwnerRoleSubstate"),
  @JsonSubTypes.Type(value = RoleAssignmentModuleRuleEntrySubstate.class, name = "RoleAssignmentModuleRuleEntry"),
  @JsonSubTypes.Type(value = RoleAssignmentModuleRuleEntrySubstate.class, name = "RoleAssignmentModuleRuleEntrySubstate"),
  @JsonSubTypes.Type(value = RoyaltyModuleFieldStateSubstate.class, name = "RoyaltyModuleFieldState"),
  @JsonSubTypes.Type(value = RoyaltyModuleFieldStateSubstate.class, name = "RoyaltyModuleFieldStateSubstate"),
  @JsonSubTypes.Type(value = RoyaltyModuleMethodRoyaltyEntrySubstate.class, name = "RoyaltyModuleMethodRoyaltyEntry"),
  @JsonSubTypes.Type(value = RoyaltyModuleMethodRoyaltyEntrySubstate.class, name = "RoyaltyModuleMethodRoyaltyEntrySubstate"),
  @JsonSubTypes.Type(value = SchemaEntrySubstate.class, name = "SchemaEntry"),
  @JsonSubTypes.Type(value = SchemaEntrySubstate.class, name = "SchemaEntrySubstate"),
  @JsonSubTypes.Type(value = TransactionTrackerCollectionEntrySubstate.class, name = "TransactionTrackerCollectionEntry"),
  @JsonSubTypes.Type(value = TransactionTrackerCollectionEntrySubstate.class, name = "TransactionTrackerCollectionEntrySubstate"),
  @JsonSubTypes.Type(value = TransactionTrackerFieldStateSubstate.class, name = "TransactionTrackerFieldState"),
  @JsonSubTypes.Type(value = TransactionTrackerFieldStateSubstate.class, name = "TransactionTrackerFieldStateSubstate"),
  @JsonSubTypes.Type(value = TwoResourcePoolFieldStateSubstate.class, name = "TwoResourcePoolFieldState"),
  @JsonSubTypes.Type(value = TwoResourcePoolFieldStateSubstate.class, name = "TwoResourcePoolFieldStateSubstate"),
  @JsonSubTypes.Type(value = TypeInfoModuleFieldTypeInfoSubstate.class, name = "TypeInfoModuleFieldTypeInfo"),
  @JsonSubTypes.Type(value = TypeInfoModuleFieldTypeInfoSubstate.class, name = "TypeInfoModuleFieldTypeInfoSubstate"),
  @JsonSubTypes.Type(value = ValidatorFieldProtocolUpdateReadinessSignalSubstate.class, name = "ValidatorFieldProtocolUpdateReadinessSignal"),
  @JsonSubTypes.Type(value = ValidatorFieldProtocolUpdateReadinessSignalSubstate.class, name = "ValidatorFieldProtocolUpdateReadinessSignalSubstate"),
  @JsonSubTypes.Type(value = ValidatorFieldStateSubstate.class, name = "ValidatorFieldState"),
  @JsonSubTypes.Type(value = ValidatorFieldStateSubstate.class, name = "ValidatorFieldStateSubstate"),
})

public class Substate {
  public static final String JSON_PROPERTY_SUBSTATE_TYPE = "substate_type";
  private SubstateType substateType;

  public static final String JSON_PROPERTY_IS_LOCKED = "is_locked";
  private Boolean isLocked;

  public Substate() { 
  }

  public Substate substateType(SubstateType substateType) {
    this.substateType = substateType;
    return this;
  }

   /**
   * Get substateType
   * @return substateType
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_SUBSTATE_TYPE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public SubstateType getSubstateType() {
    return substateType;
  }


  @JsonProperty(JSON_PROPERTY_SUBSTATE_TYPE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setSubstateType(SubstateType substateType) {
    this.substateType = substateType;
  }


  public Substate isLocked(Boolean isLocked) {
    this.isLocked = isLocked;
    return this;
  }

   /**
   * Get isLocked
   * @return isLocked
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_IS_LOCKED)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Boolean getIsLocked() {
    return isLocked;
  }


  @JsonProperty(JSON_PROPERTY_IS_LOCKED)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setIsLocked(Boolean isLocked) {
    this.isLocked = isLocked;
  }


  /**
   * Return true if this Substate object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    Substate substate = (Substate) o;
    return Objects.equals(this.substateType, substate.substateType) &&
        Objects.equals(this.isLocked, substate.isLocked);
  }

  @Override
  public int hashCode() {
    return Objects.hash(substateType, isLocked);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class Substate {\n");
    sb.append("    substateType: ").append(toIndentedString(substateType)).append("\n");
    sb.append("    isLocked: ").append(toIndentedString(isLocked)).append("\n");
    sb.append("}");
    return sb.toString();
  }

  /**
   * Convert the given object to string with each line indented by 4 spaces
   * (except the first line).
   */
  private String toIndentedString(Object o) {
    if (o == null) {
      return "null";
    }
    return o.toString().replace("\n", "\n    ");
  }

static {
  // Initialize and register the discriminator mappings.
  Map<String, Class<?>> mappings = new HashMap<String, Class<?>>();
  mappings.put("AccessControllerFieldState", AccessControllerFieldStateSubstate.class);
  mappings.put("AccessControllerFieldStateSubstate", AccessControllerFieldStateSubstate.class);
  mappings.put("AccountAuthorizedDepositorEntry", AccountAuthorizedDepositorEntrySubstate.class);
  mappings.put("AccountAuthorizedDepositorEntrySubstate", AccountAuthorizedDepositorEntrySubstate.class);
  mappings.put("AccountFieldState", AccountFieldStateSubstate.class);
  mappings.put("AccountFieldStateSubstate", AccountFieldStateSubstate.class);
  mappings.put("AccountLockerAccountClaimsEntry", AccountLockerAccountClaimsEntrySubstate.class);
  mappings.put("AccountLockerAccountClaimsEntrySubstate", AccountLockerAccountClaimsEntrySubstate.class);
  mappings.put("AccountResourcePreferenceEntry", AccountResourcePreferenceEntrySubstate.class);
  mappings.put("AccountResourcePreferenceEntrySubstate", AccountResourcePreferenceEntrySubstate.class);
  mappings.put("AccountVaultEntry", AccountVaultEntrySubstate.class);
  mappings.put("AccountVaultEntrySubstate", AccountVaultEntrySubstate.class);
  mappings.put("BootLoaderModuleFieldKernelBoot", BootLoaderModuleFieldKernelBootSubstate.class);
  mappings.put("BootLoaderModuleFieldKernelBootSubstate", BootLoaderModuleFieldKernelBootSubstate.class);
  mappings.put("BootLoaderModuleFieldSystemBoot", BootLoaderModuleFieldSystemBootSubstate.class);
  mappings.put("BootLoaderModuleFieldSystemBootSubstate", BootLoaderModuleFieldSystemBootSubstate.class);
  mappings.put("BootLoaderModuleFieldVmBoot", BootLoaderModuleFieldVmBootSubstate.class);
  mappings.put("BootLoaderModuleFieldVmBootSubstate", BootLoaderModuleFieldVmBootSubstate.class);
  mappings.put("ConsensusManagerFieldConfig", ConsensusManagerFieldConfigSubstate.class);
  mappings.put("ConsensusManagerFieldConfigSubstate", ConsensusManagerFieldConfigSubstate.class);
  mappings.put("ConsensusManagerFieldCurrentProposalStatistic", ConsensusManagerFieldCurrentProposalStatisticSubstate.class);
  mappings.put("ConsensusManagerFieldCurrentProposalStatisticSubstate", ConsensusManagerFieldCurrentProposalStatisticSubstate.class);
  mappings.put("ConsensusManagerFieldCurrentTime", ConsensusManagerFieldCurrentTimeSubstate.class);
  mappings.put("ConsensusManagerFieldCurrentTimeRoundedToMinutes", ConsensusManagerFieldCurrentTimeRoundedToMinutesSubstate.class);
  mappings.put("ConsensusManagerFieldCurrentTimeRoundedToMinutesSubstate", ConsensusManagerFieldCurrentTimeRoundedToMinutesSubstate.class);
  mappings.put("ConsensusManagerFieldCurrentTimeSubstate", ConsensusManagerFieldCurrentTimeSubstate.class);
  mappings.put("ConsensusManagerFieldCurrentValidatorSet", ConsensusManagerFieldCurrentValidatorSetSubstate.class);
  mappings.put("ConsensusManagerFieldCurrentValidatorSetSubstate", ConsensusManagerFieldCurrentValidatorSetSubstate.class);
  mappings.put("ConsensusManagerFieldState", ConsensusManagerFieldStateSubstate.class);
  mappings.put("ConsensusManagerFieldStateSubstate", ConsensusManagerFieldStateSubstate.class);
  mappings.put("ConsensusManagerFieldValidatorRewards", ConsensusManagerFieldValidatorRewardsSubstate.class);
  mappings.put("ConsensusManagerFieldValidatorRewardsSubstate", ConsensusManagerFieldValidatorRewardsSubstate.class);
  mappings.put("ConsensusManagerRegisteredValidatorsByStakeIndexEntry", ConsensusManagerRegisteredValidatorsByStakeIndexEntrySubstate.class);
  mappings.put("ConsensusManagerRegisteredValidatorsByStakeIndexEntrySubstate", ConsensusManagerRegisteredValidatorsByStakeIndexEntrySubstate.class);
  mappings.put("FungibleResourceManagerFieldDivisibility", FungibleResourceManagerFieldDivisibilitySubstate.class);
  mappings.put("FungibleResourceManagerFieldDivisibilitySubstate", FungibleResourceManagerFieldDivisibilitySubstate.class);
  mappings.put("FungibleResourceManagerFieldTotalSupply", FungibleResourceManagerFieldTotalSupplySubstate.class);
  mappings.put("FungibleResourceManagerFieldTotalSupplySubstate", FungibleResourceManagerFieldTotalSupplySubstate.class);
  mappings.put("FungibleVaultFieldBalance", FungibleVaultFieldBalanceSubstate.class);
  mappings.put("FungibleVaultFieldBalanceSubstate", FungibleVaultFieldBalanceSubstate.class);
  mappings.put("FungibleVaultFieldFrozenStatus", FungibleVaultFieldFrozenStatusSubstate.class);
  mappings.put("FungibleVaultFieldFrozenStatusSubstate", FungibleVaultFieldFrozenStatusSubstate.class);
  mappings.put("GenericKeyValueStoreEntry", GenericKeyValueStoreEntrySubstate.class);
  mappings.put("GenericKeyValueStoreEntrySubstate", GenericKeyValueStoreEntrySubstate.class);
  mappings.put("GenericScryptoComponentFieldState", GenericScryptoComponentFieldStateSubstate.class);
  mappings.put("GenericScryptoComponentFieldStateSubstate", GenericScryptoComponentFieldStateSubstate.class);
  mappings.put("MetadataModuleEntry", MetadataModuleEntrySubstate.class);
  mappings.put("MetadataModuleEntrySubstate", MetadataModuleEntrySubstate.class);
  mappings.put("MultiResourcePoolFieldState", MultiResourcePoolFieldStateSubstate.class);
  mappings.put("MultiResourcePoolFieldStateSubstate", MultiResourcePoolFieldStateSubstate.class);
  mappings.put("NonFungibleResourceManagerDataEntry", NonFungibleResourceManagerDataEntrySubstate.class);
  mappings.put("NonFungibleResourceManagerDataEntrySubstate", NonFungibleResourceManagerDataEntrySubstate.class);
  mappings.put("NonFungibleResourceManagerFieldIdType", NonFungibleResourceManagerFieldIdTypeSubstate.class);
  mappings.put("NonFungibleResourceManagerFieldIdTypeSubstate", NonFungibleResourceManagerFieldIdTypeSubstate.class);
  mappings.put("NonFungibleResourceManagerFieldMutableFields", NonFungibleResourceManagerFieldMutableFieldsSubstate.class);
  mappings.put("NonFungibleResourceManagerFieldMutableFieldsSubstate", NonFungibleResourceManagerFieldMutableFieldsSubstate.class);
  mappings.put("NonFungibleResourceManagerFieldTotalSupply", NonFungibleResourceManagerFieldTotalSupplySubstate.class);
  mappings.put("NonFungibleResourceManagerFieldTotalSupplySubstate", NonFungibleResourceManagerFieldTotalSupplySubstate.class);
  mappings.put("NonFungibleVaultContentsIndexEntry", NonFungibleVaultContentsIndexEntrySubstate.class);
  mappings.put("NonFungibleVaultContentsIndexEntrySubstate", NonFungibleVaultContentsIndexEntrySubstate.class);
  mappings.put("NonFungibleVaultFieldBalance", NonFungibleVaultFieldBalanceSubstate.class);
  mappings.put("NonFungibleVaultFieldBalanceSubstate", NonFungibleVaultFieldBalanceSubstate.class);
  mappings.put("NonFungibleVaultFieldFrozenStatus", NonFungibleVaultFieldFrozenStatusSubstate.class);
  mappings.put("NonFungibleVaultFieldFrozenStatusSubstate", NonFungibleVaultFieldFrozenStatusSubstate.class);
  mappings.put("OneResourcePoolFieldState", OneResourcePoolFieldStateSubstate.class);
  mappings.put("OneResourcePoolFieldStateSubstate", OneResourcePoolFieldStateSubstate.class);
  mappings.put("PackageBlueprintAuthTemplateEntry", PackageBlueprintAuthTemplateEntrySubstate.class);
  mappings.put("PackageBlueprintAuthTemplateEntrySubstate", PackageBlueprintAuthTemplateEntrySubstate.class);
  mappings.put("PackageBlueprintDefinitionEntry", PackageBlueprintDefinitionEntrySubstate.class);
  mappings.put("PackageBlueprintDefinitionEntrySubstate", PackageBlueprintDefinitionEntrySubstate.class);
  mappings.put("PackageBlueprintDependenciesEntry", PackageBlueprintDependenciesEntrySubstate.class);
  mappings.put("PackageBlueprintDependenciesEntrySubstate", PackageBlueprintDependenciesEntrySubstate.class);
  mappings.put("PackageBlueprintRoyaltyEntry", PackageBlueprintRoyaltyEntrySubstate.class);
  mappings.put("PackageBlueprintRoyaltyEntrySubstate", PackageBlueprintRoyaltyEntrySubstate.class);
  mappings.put("PackageCodeInstrumentedCodeEntry", PackageCodeInstrumentedCodeEntrySubstate.class);
  mappings.put("PackageCodeInstrumentedCodeEntrySubstate", PackageCodeInstrumentedCodeEntrySubstate.class);
  mappings.put("PackageCodeOriginalCodeEntry", PackageCodeOriginalCodeEntrySubstate.class);
  mappings.put("PackageCodeOriginalCodeEntrySubstate", PackageCodeOriginalCodeEntrySubstate.class);
  mappings.put("PackageCodeVmTypeEntry", PackageCodeVmTypeEntrySubstate.class);
  mappings.put("PackageCodeVmTypeEntrySubstate", PackageCodeVmTypeEntrySubstate.class);
  mappings.put("PackageFieldRoyaltyAccumulator", PackageFieldRoyaltyAccumulatorSubstate.class);
  mappings.put("PackageFieldRoyaltyAccumulatorSubstate", PackageFieldRoyaltyAccumulatorSubstate.class);
  mappings.put("RoleAssignmentModuleFieldOwnerRole", RoleAssignmentModuleFieldOwnerRoleSubstate.class);
  mappings.put("RoleAssignmentModuleFieldOwnerRoleSubstate", RoleAssignmentModuleFieldOwnerRoleSubstate.class);
  mappings.put("RoleAssignmentModuleRuleEntry", RoleAssignmentModuleRuleEntrySubstate.class);
  mappings.put("RoleAssignmentModuleRuleEntrySubstate", RoleAssignmentModuleRuleEntrySubstate.class);
  mappings.put("RoyaltyModuleFieldState", RoyaltyModuleFieldStateSubstate.class);
  mappings.put("RoyaltyModuleFieldStateSubstate", RoyaltyModuleFieldStateSubstate.class);
  mappings.put("RoyaltyModuleMethodRoyaltyEntry", RoyaltyModuleMethodRoyaltyEntrySubstate.class);
  mappings.put("RoyaltyModuleMethodRoyaltyEntrySubstate", RoyaltyModuleMethodRoyaltyEntrySubstate.class);
  mappings.put("SchemaEntry", SchemaEntrySubstate.class);
  mappings.put("SchemaEntrySubstate", SchemaEntrySubstate.class);
  mappings.put("TransactionTrackerCollectionEntry", TransactionTrackerCollectionEntrySubstate.class);
  mappings.put("TransactionTrackerCollectionEntrySubstate", TransactionTrackerCollectionEntrySubstate.class);
  mappings.put("TransactionTrackerFieldState", TransactionTrackerFieldStateSubstate.class);
  mappings.put("TransactionTrackerFieldStateSubstate", TransactionTrackerFieldStateSubstate.class);
  mappings.put("TwoResourcePoolFieldState", TwoResourcePoolFieldStateSubstate.class);
  mappings.put("TwoResourcePoolFieldStateSubstate", TwoResourcePoolFieldStateSubstate.class);
  mappings.put("TypeInfoModuleFieldTypeInfo", TypeInfoModuleFieldTypeInfoSubstate.class);
  mappings.put("TypeInfoModuleFieldTypeInfoSubstate", TypeInfoModuleFieldTypeInfoSubstate.class);
  mappings.put("ValidatorFieldProtocolUpdateReadinessSignal", ValidatorFieldProtocolUpdateReadinessSignalSubstate.class);
  mappings.put("ValidatorFieldProtocolUpdateReadinessSignalSubstate", ValidatorFieldProtocolUpdateReadinessSignalSubstate.class);
  mappings.put("ValidatorFieldState", ValidatorFieldStateSubstate.class);
  mappings.put("ValidatorFieldStateSubstate", ValidatorFieldStateSubstate.class);
  mappings.put("Substate", Substate.class);
  JSON.registerDiscriminator(Substate.class, "substate_type", mappings);
}
}

