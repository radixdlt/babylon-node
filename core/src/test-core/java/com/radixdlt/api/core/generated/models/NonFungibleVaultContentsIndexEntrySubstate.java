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
import com.radixdlt.api.core.generated.models.AccessRulesModuleFieldAccessRulesSubstate;
import com.radixdlt.api.core.generated.models.AccountVaultIndexEntrySubstate;
import com.radixdlt.api.core.generated.models.ClockFieldStateSubstate;
import com.radixdlt.api.core.generated.models.EpochManagerFieldConfigSubstate;
import com.radixdlt.api.core.generated.models.EpochManagerFieldCurrentValidatorSetSubstate;
import com.radixdlt.api.core.generated.models.EpochManagerFieldStateSubstate;
import com.radixdlt.api.core.generated.models.EpochManagerRegisteredValidatorsByStakeIndexEntrySubstate;
import com.radixdlt.api.core.generated.models.FungibleResourceManagerFieldDivisibilitySubstate;
import com.radixdlt.api.core.generated.models.FungibleResourceManagerFieldTotalSupplySubstate;
import com.radixdlt.api.core.generated.models.FungibleVaultFieldBalanceSubstate;
import com.radixdlt.api.core.generated.models.GenericKeyValueStoreEntrySubstate;
import com.radixdlt.api.core.generated.models.GenericScryptoComponentFieldStateSubstate;
import com.radixdlt.api.core.generated.models.MetadataModuleEntrySubstate;
import com.radixdlt.api.core.generated.models.NonFungibleLocalId;
import com.radixdlt.api.core.generated.models.NonFungibleResourceManagerDataEntrySubstate;
import com.radixdlt.api.core.generated.models.NonFungibleResourceManagerFieldIdTypeSubstate;
import com.radixdlt.api.core.generated.models.NonFungibleResourceManagerFieldMutableFieldsSubstate;
import com.radixdlt.api.core.generated.models.NonFungibleResourceManagerFieldTotalSupplySubstate;
import com.radixdlt.api.core.generated.models.NonFungibleVaultContentsIndexEntrySubstate;
import com.radixdlt.api.core.generated.models.NonFungibleVaultContentsIndexEntrySubstateAllOf;
import com.radixdlt.api.core.generated.models.NonFungibleVaultFieldBalanceSubstate;
import com.radixdlt.api.core.generated.models.PackageFieldCodeSubstate;
import com.radixdlt.api.core.generated.models.PackageFieldCodeTypeSubstate;
import com.radixdlt.api.core.generated.models.PackageFieldFunctionAccessRulesSubstate;
import com.radixdlt.api.core.generated.models.PackageFieldInfoSubstate;
import com.radixdlt.api.core.generated.models.PackageFieldRoyaltySubstate;
import com.radixdlt.api.core.generated.models.RoyaltyModuleFieldAccumulatorSubstate;
import com.radixdlt.api.core.generated.models.RoyaltyModuleFieldConfigSubstate;
import com.radixdlt.api.core.generated.models.Substate;
import com.radixdlt.api.core.generated.models.SubstateType;
import com.radixdlt.api.core.generated.models.TypeInfoModuleFieldTypeInfoSubstate;
import com.radixdlt.api.core.generated.models.ValidatorFieldStateSubstate;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


import com.radixdlt.api.core.generated.client.JSON;
/**
 * NonFungibleVaultContentsIndexEntrySubstate
 */
@JsonPropertyOrder({
  NonFungibleVaultContentsIndexEntrySubstate.JSON_PROPERTY_NON_FUNGIBLE_LOCAL_ID
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
@JsonIgnoreProperties(
  value = "substate_type", // ignore manually set substate_type, it will be automatically generated by Jackson during serialization
  allowSetters = true // allows the substate_type to be set during deserialization
)
@JsonTypeInfo(use = JsonTypeInfo.Id.NAME, include = JsonTypeInfo.As.PROPERTY, property = "substate_type", visible = true)
@JsonSubTypes({
  @JsonSubTypes.Type(value = AccessControllerFieldStateSubstate.class, name = "AccessControllerFieldState"),
  @JsonSubTypes.Type(value = AccessRulesModuleFieldAccessRulesSubstate.class, name = "AccessRulesModuleFieldAccessRules"),
  @JsonSubTypes.Type(value = AccountVaultIndexEntrySubstate.class, name = "AccountVaultIndexEntry"),
  @JsonSubTypes.Type(value = ClockFieldStateSubstate.class, name = "ClockFieldState"),
  @JsonSubTypes.Type(value = EpochManagerFieldConfigSubstate.class, name = "EpochManagerFieldConfig"),
  @JsonSubTypes.Type(value = EpochManagerFieldCurrentValidatorSetSubstate.class, name = "EpochManagerFieldCurrentValidatorSet"),
  @JsonSubTypes.Type(value = EpochManagerFieldStateSubstate.class, name = "EpochManagerFieldState"),
  @JsonSubTypes.Type(value = EpochManagerRegisteredValidatorsByStakeIndexEntrySubstate.class, name = "EpochManagerRegisteredValidatorsByStakeIndexEntry"),
  @JsonSubTypes.Type(value = FungibleResourceManagerFieldDivisibilitySubstate.class, name = "FungibleResourceManagerFieldDivisibility"),
  @JsonSubTypes.Type(value = FungibleResourceManagerFieldTotalSupplySubstate.class, name = "FungibleResourceManagerFieldTotalSupply"),
  @JsonSubTypes.Type(value = FungibleVaultFieldBalanceSubstate.class, name = "FungibleVaultFieldBalance"),
  @JsonSubTypes.Type(value = GenericKeyValueStoreEntrySubstate.class, name = "GenericKeyValueStoreEntry"),
  @JsonSubTypes.Type(value = GenericScryptoComponentFieldStateSubstate.class, name = "GenericScryptoComponentFieldState"),
  @JsonSubTypes.Type(value = MetadataModuleEntrySubstate.class, name = "MetadataModuleEntry"),
  @JsonSubTypes.Type(value = NonFungibleResourceManagerDataEntrySubstate.class, name = "NonFungibleResourceManagerDataEntry"),
  @JsonSubTypes.Type(value = NonFungibleResourceManagerFieldIdTypeSubstate.class, name = "NonFungibleResourceManagerFieldIdType"),
  @JsonSubTypes.Type(value = NonFungibleResourceManagerFieldMutableFieldsSubstate.class, name = "NonFungibleResourceManagerFieldMutableFields"),
  @JsonSubTypes.Type(value = NonFungibleResourceManagerFieldTotalSupplySubstate.class, name = "NonFungibleResourceManagerFieldTotalSupply"),
  @JsonSubTypes.Type(value = NonFungibleVaultContentsIndexEntrySubstate.class, name = "NonFungibleVaultContentsIndexEntry"),
  @JsonSubTypes.Type(value = NonFungibleVaultFieldBalanceSubstate.class, name = "NonFungibleVaultFieldBalance"),
  @JsonSubTypes.Type(value = PackageFieldCodeSubstate.class, name = "PackageFieldCode"),
  @JsonSubTypes.Type(value = PackageFieldCodeTypeSubstate.class, name = "PackageFieldCodeType"),
  @JsonSubTypes.Type(value = PackageFieldFunctionAccessRulesSubstate.class, name = "PackageFieldFunctionAccessRules"),
  @JsonSubTypes.Type(value = PackageFieldInfoSubstate.class, name = "PackageFieldInfo"),
  @JsonSubTypes.Type(value = PackageFieldRoyaltySubstate.class, name = "PackageFieldRoyalty"),
  @JsonSubTypes.Type(value = RoyaltyModuleFieldAccumulatorSubstate.class, name = "RoyaltyModuleFieldAccumulator"),
  @JsonSubTypes.Type(value = RoyaltyModuleFieldConfigSubstate.class, name = "RoyaltyModuleFieldConfig"),
  @JsonSubTypes.Type(value = TypeInfoModuleFieldTypeInfoSubstate.class, name = "TypeInfoModuleFieldTypeInfo"),
  @JsonSubTypes.Type(value = ValidatorFieldStateSubstate.class, name = "ValidatorFieldState"),
})

public class NonFungibleVaultContentsIndexEntrySubstate extends Substate {
  public static final String JSON_PROPERTY_NON_FUNGIBLE_LOCAL_ID = "non_fungible_local_id";
  private NonFungibleLocalId nonFungibleLocalId;

  public NonFungibleVaultContentsIndexEntrySubstate() { 
  }

  public NonFungibleVaultContentsIndexEntrySubstate nonFungibleLocalId(NonFungibleLocalId nonFungibleLocalId) {
    this.nonFungibleLocalId = nonFungibleLocalId;
    return this;
  }

   /**
   * Get nonFungibleLocalId
   * @return nonFungibleLocalId
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_NON_FUNGIBLE_LOCAL_ID)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public NonFungibleLocalId getNonFungibleLocalId() {
    return nonFungibleLocalId;
  }


  @JsonProperty(JSON_PROPERTY_NON_FUNGIBLE_LOCAL_ID)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setNonFungibleLocalId(NonFungibleLocalId nonFungibleLocalId) {
    this.nonFungibleLocalId = nonFungibleLocalId;
  }


  /**
   * Return true if this NonFungibleVaultContentsIndexEntrySubstate object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    NonFungibleVaultContentsIndexEntrySubstate nonFungibleVaultContentsIndexEntrySubstate = (NonFungibleVaultContentsIndexEntrySubstate) o;
    return Objects.equals(this.nonFungibleLocalId, nonFungibleVaultContentsIndexEntrySubstate.nonFungibleLocalId) &&
        super.equals(o);
  }

  @Override
  public int hashCode() {
    return Objects.hash(nonFungibleLocalId, super.hashCode());
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class NonFungibleVaultContentsIndexEntrySubstate {\n");
    sb.append("    ").append(toIndentedString(super.toString())).append("\n");
    sb.append("    nonFungibleLocalId: ").append(toIndentedString(nonFungibleLocalId)).append("\n");
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
  mappings.put("AccessRulesModuleFieldAccessRules", AccessRulesModuleFieldAccessRulesSubstate.class);
  mappings.put("AccountVaultIndexEntry", AccountVaultIndexEntrySubstate.class);
  mappings.put("ClockFieldState", ClockFieldStateSubstate.class);
  mappings.put("EpochManagerFieldConfig", EpochManagerFieldConfigSubstate.class);
  mappings.put("EpochManagerFieldCurrentValidatorSet", EpochManagerFieldCurrentValidatorSetSubstate.class);
  mappings.put("EpochManagerFieldState", EpochManagerFieldStateSubstate.class);
  mappings.put("EpochManagerRegisteredValidatorsByStakeIndexEntry", EpochManagerRegisteredValidatorsByStakeIndexEntrySubstate.class);
  mappings.put("FungibleResourceManagerFieldDivisibility", FungibleResourceManagerFieldDivisibilitySubstate.class);
  mappings.put("FungibleResourceManagerFieldTotalSupply", FungibleResourceManagerFieldTotalSupplySubstate.class);
  mappings.put("FungibleVaultFieldBalance", FungibleVaultFieldBalanceSubstate.class);
  mappings.put("GenericKeyValueStoreEntry", GenericKeyValueStoreEntrySubstate.class);
  mappings.put("GenericScryptoComponentFieldState", GenericScryptoComponentFieldStateSubstate.class);
  mappings.put("MetadataModuleEntry", MetadataModuleEntrySubstate.class);
  mappings.put("NonFungibleResourceManagerDataEntry", NonFungibleResourceManagerDataEntrySubstate.class);
  mappings.put("NonFungibleResourceManagerFieldIdType", NonFungibleResourceManagerFieldIdTypeSubstate.class);
  mappings.put("NonFungibleResourceManagerFieldMutableFields", NonFungibleResourceManagerFieldMutableFieldsSubstate.class);
  mappings.put("NonFungibleResourceManagerFieldTotalSupply", NonFungibleResourceManagerFieldTotalSupplySubstate.class);
  mappings.put("NonFungibleVaultContentsIndexEntry", NonFungibleVaultContentsIndexEntrySubstate.class);
  mappings.put("NonFungibleVaultFieldBalance", NonFungibleVaultFieldBalanceSubstate.class);
  mappings.put("PackageFieldCode", PackageFieldCodeSubstate.class);
  mappings.put("PackageFieldCodeType", PackageFieldCodeTypeSubstate.class);
  mappings.put("PackageFieldFunctionAccessRules", PackageFieldFunctionAccessRulesSubstate.class);
  mappings.put("PackageFieldInfo", PackageFieldInfoSubstate.class);
  mappings.put("PackageFieldRoyalty", PackageFieldRoyaltySubstate.class);
  mappings.put("RoyaltyModuleFieldAccumulator", RoyaltyModuleFieldAccumulatorSubstate.class);
  mappings.put("RoyaltyModuleFieldConfig", RoyaltyModuleFieldConfigSubstate.class);
  mappings.put("TypeInfoModuleFieldTypeInfo", TypeInfoModuleFieldTypeInfoSubstate.class);
  mappings.put("ValidatorFieldState", ValidatorFieldStateSubstate.class);
  mappings.put("NonFungibleVaultContentsIndexEntrySubstate", NonFungibleVaultContentsIndexEntrySubstate.class);
  JSON.registerDiscriminator(NonFungibleVaultContentsIndexEntrySubstate.class, "substate_type", mappings);
}
}

