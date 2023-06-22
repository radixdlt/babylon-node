/*
 * Babylon Core API - RCnet V2
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  This version of the Core API belongs to the first release candidate of the Radix Babylon network (\"RCnet-V1\").  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` are guaranteed to be forward compatible to Babylon mainnet launch (and beyond). We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  We give no guarantees that other endpoints will not change before Babylon mainnet launch, although changes are expected to be minimal. 
 *
 * The version of the OpenAPI document: 0.4.0
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
import com.radixdlt.api.core.generated.models.AccessRuleEntrySubstate;
import com.radixdlt.api.core.generated.models.AccessRulesModuleFieldAccessRulesSubstate;
import com.radixdlt.api.core.generated.models.AccountDepositRuleIndexEntrySubstate;
import com.radixdlt.api.core.generated.models.AccountFieldStateSubstate;
import com.radixdlt.api.core.generated.models.AccountVaultIndexEntrySubstate;
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
import com.radixdlt.api.core.generated.models.GenericKeyValueStoreEntrySubstate;
import com.radixdlt.api.core.generated.models.GenericScryptoComponentFieldStateSubstate;
import com.radixdlt.api.core.generated.models.MetadataModuleEntrySubstate;
import com.radixdlt.api.core.generated.models.MultiResourcePoolSubstate;
import com.radixdlt.api.core.generated.models.MutabilityEntrySubstate;
import com.radixdlt.api.core.generated.models.NonFungibleResourceManagerDataEntrySubstate;
import com.radixdlt.api.core.generated.models.NonFungibleResourceManagerFieldIdTypeSubstate;
import com.radixdlt.api.core.generated.models.NonFungibleResourceManagerFieldMutableFieldsSubstate;
import com.radixdlt.api.core.generated.models.NonFungibleResourceManagerFieldTotalSupplySubstate;
import com.radixdlt.api.core.generated.models.NonFungibleVaultContentsIndexEntrySubstate;
import com.radixdlt.api.core.generated.models.NonFungibleVaultFieldBalanceSubstate;
import com.radixdlt.api.core.generated.models.OneResourcePoolSubstate;
import com.radixdlt.api.core.generated.models.OwnerRoleSubstate;
import com.radixdlt.api.core.generated.models.PackageAuthTemplateEntrySubstate;
import com.radixdlt.api.core.generated.models.PackageBlueprintDependenciesEntrySubstate;
import com.radixdlt.api.core.generated.models.PackageBlueprintEntrySubstate;
import com.radixdlt.api.core.generated.models.PackageCodeEntrySubstate;
import com.radixdlt.api.core.generated.models.PackageCodeSubstate;
import com.radixdlt.api.core.generated.models.PackageFieldFunctionAccessRulesSubstate;
import com.radixdlt.api.core.generated.models.PackageFieldInfoSubstate;
import com.radixdlt.api.core.generated.models.PackageFieldRoyaltyAccumulatorSubstate;
import com.radixdlt.api.core.generated.models.PackageRoyaltyEntrySubstate;
import com.radixdlt.api.core.generated.models.PackageSchemaEntrySubstate;
import com.radixdlt.api.core.generated.models.PackageSchemaEntrySubstateAllOf;
import com.radixdlt.api.core.generated.models.RoyaltyModuleFieldAccumulatorSubstate;
import com.radixdlt.api.core.generated.models.RoyaltyModuleFieldConfigSubstate;
import com.radixdlt.api.core.generated.models.ScryptoSchema;
import com.radixdlt.api.core.generated.models.Substate;
import com.radixdlt.api.core.generated.models.SubstateType;
import com.radixdlt.api.core.generated.models.TransactionTrackerCollectionEntrySubstate;
import com.radixdlt.api.core.generated.models.TransactionTrackerSubstate;
import com.radixdlt.api.core.generated.models.TwoResourcePoolSubstate;
import com.radixdlt.api.core.generated.models.TypeInfoModuleFieldTypeInfoSubstate;
import com.radixdlt.api.core.generated.models.ValidatorFieldStateSubstate;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


import com.radixdlt.api.core.generated.client.JSON;
/**
 * PackageSchemaEntrySubstate
 */
@JsonPropertyOrder({
  PackageSchemaEntrySubstate.JSON_PROPERTY_SCHEMA_HASH,
  PackageSchemaEntrySubstate.JSON_PROPERTY_SCHEMA
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
@JsonIgnoreProperties(
  value = "substate_type", // ignore manually set substate_type, it will be automatically generated by Jackson during serialization
  allowSetters = true // allows the substate_type to be set during deserialization
)
@JsonTypeInfo(use = JsonTypeInfo.Id.NAME, include = JsonTypeInfo.As.PROPERTY, property = "substate_type", visible = true)
@JsonSubTypes({
  @JsonSubTypes.Type(value = AccessControllerFieldStateSubstate.class, name = "AccessControllerFieldState"),
  @JsonSubTypes.Type(value = AccessRuleEntrySubstate.class, name = "AccessRuleEntry"),
  @JsonSubTypes.Type(value = AccessRulesModuleFieldAccessRulesSubstate.class, name = "AccessRulesModuleFieldAccessRules"),
  @JsonSubTypes.Type(value = AccountDepositRuleIndexEntrySubstate.class, name = "AccountDepositRuleIndexEntry"),
  @JsonSubTypes.Type(value = AccountFieldStateSubstate.class, name = "AccountFieldState"),
  @JsonSubTypes.Type(value = AccountVaultIndexEntrySubstate.class, name = "AccountVaultIndexEntry"),
  @JsonSubTypes.Type(value = ConsensusManagerFieldConfigSubstate.class, name = "ConsensusManagerFieldConfig"),
  @JsonSubTypes.Type(value = ConsensusManagerFieldCurrentProposalStatisticSubstate.class, name = "ConsensusManagerFieldCurrentProposalStatistic"),
  @JsonSubTypes.Type(value = ConsensusManagerFieldCurrentTimeSubstate.class, name = "ConsensusManagerFieldCurrentTime"),
  @JsonSubTypes.Type(value = ConsensusManagerFieldCurrentTimeRoundedToMinutesSubstate.class, name = "ConsensusManagerFieldCurrentTimeRoundedToMinutes"),
  @JsonSubTypes.Type(value = ConsensusManagerFieldCurrentValidatorSetSubstate.class, name = "ConsensusManagerFieldCurrentValidatorSet"),
  @JsonSubTypes.Type(value = ConsensusManagerFieldStateSubstate.class, name = "ConsensusManagerFieldState"),
  @JsonSubTypes.Type(value = ConsensusManagerFieldValidatorRewardsSubstate.class, name = "ConsensusManagerFieldValidatorRewards"),
  @JsonSubTypes.Type(value = ConsensusManagerRegisteredValidatorsByStakeIndexEntrySubstate.class, name = "ConsensusManagerRegisteredValidatorsByStakeIndexEntry"),
  @JsonSubTypes.Type(value = FungibleResourceManagerFieldDivisibilitySubstate.class, name = "FungibleResourceManagerFieldDivisibility"),
  @JsonSubTypes.Type(value = FungibleResourceManagerFieldTotalSupplySubstate.class, name = "FungibleResourceManagerFieldTotalSupply"),
  @JsonSubTypes.Type(value = FungibleVaultFieldBalanceSubstate.class, name = "FungibleVaultFieldBalance"),
  @JsonSubTypes.Type(value = GenericKeyValueStoreEntrySubstate.class, name = "GenericKeyValueStoreEntry"),
  @JsonSubTypes.Type(value = GenericScryptoComponentFieldStateSubstate.class, name = "GenericScryptoComponentFieldState"),
  @JsonSubTypes.Type(value = MetadataModuleEntrySubstate.class, name = "MetadataModuleEntry"),
  @JsonSubTypes.Type(value = MultiResourcePoolSubstate.class, name = "MultiResourcePool"),
  @JsonSubTypes.Type(value = MutabilityEntrySubstate.class, name = "MutabilityEntry"),
  @JsonSubTypes.Type(value = NonFungibleResourceManagerDataEntrySubstate.class, name = "NonFungibleResourceManagerDataEntry"),
  @JsonSubTypes.Type(value = NonFungibleResourceManagerFieldIdTypeSubstate.class, name = "NonFungibleResourceManagerFieldIdType"),
  @JsonSubTypes.Type(value = NonFungibleResourceManagerFieldMutableFieldsSubstate.class, name = "NonFungibleResourceManagerFieldMutableFields"),
  @JsonSubTypes.Type(value = NonFungibleResourceManagerFieldTotalSupplySubstate.class, name = "NonFungibleResourceManagerFieldTotalSupply"),
  @JsonSubTypes.Type(value = NonFungibleVaultContentsIndexEntrySubstate.class, name = "NonFungibleVaultContentsIndexEntry"),
  @JsonSubTypes.Type(value = NonFungibleVaultFieldBalanceSubstate.class, name = "NonFungibleVaultFieldBalance"),
  @JsonSubTypes.Type(value = OneResourcePoolSubstate.class, name = "OneResourcePool"),
  @JsonSubTypes.Type(value = OwnerRoleSubstate.class, name = "OwnerRole"),
  @JsonSubTypes.Type(value = PackageAuthTemplateEntrySubstate.class, name = "PackageAuthTemplateEntry"),
  @JsonSubTypes.Type(value = PackageBlueprintDependenciesEntrySubstate.class, name = "PackageBlueprintDependenciesEntry"),
  @JsonSubTypes.Type(value = PackageBlueprintEntrySubstate.class, name = "PackageBlueprintEntry"),
  @JsonSubTypes.Type(value = PackageCodeSubstate.class, name = "PackageCode"),
  @JsonSubTypes.Type(value = PackageCodeEntrySubstate.class, name = "PackageCodeEntry"),
  @JsonSubTypes.Type(value = PackageFieldFunctionAccessRulesSubstate.class, name = "PackageFieldFunctionAccessRules"),
  @JsonSubTypes.Type(value = PackageFieldInfoSubstate.class, name = "PackageFieldInfo"),
  @JsonSubTypes.Type(value = PackageFieldRoyaltyAccumulatorSubstate.class, name = "PackageFieldRoyaltyAccumulator"),
  @JsonSubTypes.Type(value = PackageRoyaltyEntrySubstate.class, name = "PackageRoyaltyEntry"),
  @JsonSubTypes.Type(value = PackageSchemaEntrySubstate.class, name = "PackageSchemaEntry"),
  @JsonSubTypes.Type(value = RoyaltyModuleFieldAccumulatorSubstate.class, name = "RoyaltyModuleFieldAccumulator"),
  @JsonSubTypes.Type(value = RoyaltyModuleFieldConfigSubstate.class, name = "RoyaltyModuleFieldConfig"),
  @JsonSubTypes.Type(value = TransactionTrackerSubstate.class, name = "TransactionTracker"),
  @JsonSubTypes.Type(value = TransactionTrackerCollectionEntrySubstate.class, name = "TransactionTrackerCollectionEntry"),
  @JsonSubTypes.Type(value = TwoResourcePoolSubstate.class, name = "TwoResourcePool"),
  @JsonSubTypes.Type(value = TypeInfoModuleFieldTypeInfoSubstate.class, name = "TypeInfoModuleFieldTypeInfo"),
  @JsonSubTypes.Type(value = ValidatorFieldStateSubstate.class, name = "ValidatorFieldState"),
})

public class PackageSchemaEntrySubstate extends Substate {
  public static final String JSON_PROPERTY_SCHEMA_HASH = "schema_hash";
  private String schemaHash;

  public static final String JSON_PROPERTY_SCHEMA = "schema";
  private ScryptoSchema schema;

  public PackageSchemaEntrySubstate() { 
  }

  public PackageSchemaEntrySubstate schemaHash(String schemaHash) {
    this.schemaHash = schemaHash;
    return this;
  }

   /**
   * The hex-encoded schema hash.
   * @return schemaHash
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The hex-encoded schema hash.")
  @JsonProperty(JSON_PROPERTY_SCHEMA_HASH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getSchemaHash() {
    return schemaHash;
  }


  @JsonProperty(JSON_PROPERTY_SCHEMA_HASH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setSchemaHash(String schemaHash) {
    this.schemaHash = schemaHash;
  }


  public PackageSchemaEntrySubstate schema(ScryptoSchema schema) {
    this.schema = schema;
    return this;
  }

   /**
   * Get schema
   * @return schema
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "")
  @JsonProperty(JSON_PROPERTY_SCHEMA)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public ScryptoSchema getSchema() {
    return schema;
  }


  @JsonProperty(JSON_PROPERTY_SCHEMA)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setSchema(ScryptoSchema schema) {
    this.schema = schema;
  }


  /**
   * Return true if this PackageSchemaEntrySubstate object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    PackageSchemaEntrySubstate packageSchemaEntrySubstate = (PackageSchemaEntrySubstate) o;
    return Objects.equals(this.schemaHash, packageSchemaEntrySubstate.schemaHash) &&
        Objects.equals(this.schema, packageSchemaEntrySubstate.schema) &&
        super.equals(o);
  }

  @Override
  public int hashCode() {
    return Objects.hash(schemaHash, schema, super.hashCode());
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class PackageSchemaEntrySubstate {\n");
    sb.append("    ").append(toIndentedString(super.toString())).append("\n");
    sb.append("    schemaHash: ").append(toIndentedString(schemaHash)).append("\n");
    sb.append("    schema: ").append(toIndentedString(schema)).append("\n");
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
  mappings.put("AccessRuleEntry", AccessRuleEntrySubstate.class);
  mappings.put("AccessRulesModuleFieldAccessRules", AccessRulesModuleFieldAccessRulesSubstate.class);
  mappings.put("AccountDepositRuleIndexEntry", AccountDepositRuleIndexEntrySubstate.class);
  mappings.put("AccountFieldState", AccountFieldStateSubstate.class);
  mappings.put("AccountVaultIndexEntry", AccountVaultIndexEntrySubstate.class);
  mappings.put("ConsensusManagerFieldConfig", ConsensusManagerFieldConfigSubstate.class);
  mappings.put("ConsensusManagerFieldCurrentProposalStatistic", ConsensusManagerFieldCurrentProposalStatisticSubstate.class);
  mappings.put("ConsensusManagerFieldCurrentTime", ConsensusManagerFieldCurrentTimeSubstate.class);
  mappings.put("ConsensusManagerFieldCurrentTimeRoundedToMinutes", ConsensusManagerFieldCurrentTimeRoundedToMinutesSubstate.class);
  mappings.put("ConsensusManagerFieldCurrentValidatorSet", ConsensusManagerFieldCurrentValidatorSetSubstate.class);
  mappings.put("ConsensusManagerFieldState", ConsensusManagerFieldStateSubstate.class);
  mappings.put("ConsensusManagerFieldValidatorRewards", ConsensusManagerFieldValidatorRewardsSubstate.class);
  mappings.put("ConsensusManagerRegisteredValidatorsByStakeIndexEntry", ConsensusManagerRegisteredValidatorsByStakeIndexEntrySubstate.class);
  mappings.put("FungibleResourceManagerFieldDivisibility", FungibleResourceManagerFieldDivisibilitySubstate.class);
  mappings.put("FungibleResourceManagerFieldTotalSupply", FungibleResourceManagerFieldTotalSupplySubstate.class);
  mappings.put("FungibleVaultFieldBalance", FungibleVaultFieldBalanceSubstate.class);
  mappings.put("GenericKeyValueStoreEntry", GenericKeyValueStoreEntrySubstate.class);
  mappings.put("GenericScryptoComponentFieldState", GenericScryptoComponentFieldStateSubstate.class);
  mappings.put("MetadataModuleEntry", MetadataModuleEntrySubstate.class);
  mappings.put("MultiResourcePool", MultiResourcePoolSubstate.class);
  mappings.put("MutabilityEntry", MutabilityEntrySubstate.class);
  mappings.put("NonFungibleResourceManagerDataEntry", NonFungibleResourceManagerDataEntrySubstate.class);
  mappings.put("NonFungibleResourceManagerFieldIdType", NonFungibleResourceManagerFieldIdTypeSubstate.class);
  mappings.put("NonFungibleResourceManagerFieldMutableFields", NonFungibleResourceManagerFieldMutableFieldsSubstate.class);
  mappings.put("NonFungibleResourceManagerFieldTotalSupply", NonFungibleResourceManagerFieldTotalSupplySubstate.class);
  mappings.put("NonFungibleVaultContentsIndexEntry", NonFungibleVaultContentsIndexEntrySubstate.class);
  mappings.put("NonFungibleVaultFieldBalance", NonFungibleVaultFieldBalanceSubstate.class);
  mappings.put("OneResourcePool", OneResourcePoolSubstate.class);
  mappings.put("OwnerRole", OwnerRoleSubstate.class);
  mappings.put("PackageAuthTemplateEntry", PackageAuthTemplateEntrySubstate.class);
  mappings.put("PackageBlueprintDependenciesEntry", PackageBlueprintDependenciesEntrySubstate.class);
  mappings.put("PackageBlueprintEntry", PackageBlueprintEntrySubstate.class);
  mappings.put("PackageCode", PackageCodeSubstate.class);
  mappings.put("PackageCodeEntry", PackageCodeEntrySubstate.class);
  mappings.put("PackageFieldFunctionAccessRules", PackageFieldFunctionAccessRulesSubstate.class);
  mappings.put("PackageFieldInfo", PackageFieldInfoSubstate.class);
  mappings.put("PackageFieldRoyaltyAccumulator", PackageFieldRoyaltyAccumulatorSubstate.class);
  mappings.put("PackageRoyaltyEntry", PackageRoyaltyEntrySubstate.class);
  mappings.put("PackageSchemaEntry", PackageSchemaEntrySubstate.class);
  mappings.put("RoyaltyModuleFieldAccumulator", RoyaltyModuleFieldAccumulatorSubstate.class);
  mappings.put("RoyaltyModuleFieldConfig", RoyaltyModuleFieldConfigSubstate.class);
  mappings.put("TransactionTracker", TransactionTrackerSubstate.class);
  mappings.put("TransactionTrackerCollectionEntry", TransactionTrackerCollectionEntrySubstate.class);
  mappings.put("TwoResourcePool", TwoResourcePoolSubstate.class);
  mappings.put("TypeInfoModuleFieldTypeInfo", TypeInfoModuleFieldTypeInfoSubstate.class);
  mappings.put("ValidatorFieldState", ValidatorFieldStateSubstate.class);
  mappings.put("PackageSchemaEntrySubstate", PackageSchemaEntrySubstate.class);
  JSON.registerDiscriminator(PackageSchemaEntrySubstate.class, "substate_type", mappings);
}
}

