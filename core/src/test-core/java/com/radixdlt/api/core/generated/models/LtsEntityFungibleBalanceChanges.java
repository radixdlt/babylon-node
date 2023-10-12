/*
 * Radix Core API - Babylon
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.0.4
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
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonTypeName;
import com.fasterxml.jackson.annotation.JsonValue;
import com.radixdlt.api.core.generated.models.LtsFeeFungibleResourceBalanceChange;
import com.radixdlt.api.core.generated.models.LtsFungibleResourceBalanceChange;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import java.util.ArrayList;
import java.util.List;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * LtsEntityFungibleBalanceChanges
 */
@JsonPropertyOrder({
  LtsEntityFungibleBalanceChanges.JSON_PROPERTY_ENTITY_ADDRESS,
  LtsEntityFungibleBalanceChanges.JSON_PROPERTY_FEE_BALANCE_CHANGE,
  LtsEntityFungibleBalanceChanges.JSON_PROPERTY_FEE_BALANCE_CHANGES,
  LtsEntityFungibleBalanceChanges.JSON_PROPERTY_NON_FEE_BALANCE_CHANGES
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class LtsEntityFungibleBalanceChanges {
  public static final String JSON_PROPERTY_ENTITY_ADDRESS = "entity_address";
  private String entityAddress;

  public static final String JSON_PROPERTY_FEE_BALANCE_CHANGE = "fee_balance_change";
  private LtsFungibleResourceBalanceChange feeBalanceChange;

  public static final String JSON_PROPERTY_FEE_BALANCE_CHANGES = "fee_balance_changes";
  private List<LtsFeeFungibleResourceBalanceChange> feeBalanceChanges = new ArrayList<>();

  public static final String JSON_PROPERTY_NON_FEE_BALANCE_CHANGES = "non_fee_balance_changes";
  private List<LtsFungibleResourceBalanceChange> nonFeeBalanceChanges = new ArrayList<>();

  public LtsEntityFungibleBalanceChanges() { 
  }

  public LtsEntityFungibleBalanceChanges entityAddress(String entityAddress) {
    this.entityAddress = entityAddress;
    return this;
  }

   /**
   * The Bech32m-encoded human readable version of the entity&#39;s address
   * @return entityAddress
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The Bech32m-encoded human readable version of the entity's address")
  @JsonProperty(JSON_PROPERTY_ENTITY_ADDRESS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getEntityAddress() {
    return entityAddress;
  }


  @JsonProperty(JSON_PROPERTY_ENTITY_ADDRESS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setEntityAddress(String entityAddress) {
    this.entityAddress = entityAddress;
  }


  public LtsEntityFungibleBalanceChanges feeBalanceChange(LtsFungibleResourceBalanceChange feeBalanceChange) {
    this.feeBalanceChange = feeBalanceChange;
    return this;
  }

   /**
   * Get feeBalanceChange
   * @return feeBalanceChange
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "")
  @JsonProperty(JSON_PROPERTY_FEE_BALANCE_CHANGE)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public LtsFungibleResourceBalanceChange getFeeBalanceChange() {
    return feeBalanceChange;
  }


  @JsonProperty(JSON_PROPERTY_FEE_BALANCE_CHANGE)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setFeeBalanceChange(LtsFungibleResourceBalanceChange feeBalanceChange) {
    this.feeBalanceChange = feeBalanceChange;
  }


  public LtsEntityFungibleBalanceChanges feeBalanceChanges(List<LtsFeeFungibleResourceBalanceChange> feeBalanceChanges) {
    this.feeBalanceChanges = feeBalanceChanges;
    return this;
  }

  public LtsEntityFungibleBalanceChanges addFeeBalanceChangesItem(LtsFeeFungibleResourceBalanceChange feeBalanceChangesItem) {
    this.feeBalanceChanges.add(feeBalanceChangesItem);
    return this;
  }

   /**
   * If present, this field indicates fee-related balance changes, for example:  - Payment of the fee (including tip and royalty) - Distribution of royalties - Distribution of the fee and tip to the consensus-manager, for distributing to the relevant   validator/s at end of epoch  See https://www.radixdlt.com/blog/how-fees-work-in-babylon for further information on how fee payment works at Babylon. 
   * @return feeBalanceChanges
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "If present, this field indicates fee-related balance changes, for example:  - Payment of the fee (including tip and royalty) - Distribution of royalties - Distribution of the fee and tip to the consensus-manager, for distributing to the relevant   validator/s at end of epoch  See https://www.radixdlt.com/blog/how-fees-work-in-babylon for further information on how fee payment works at Babylon. ")
  @JsonProperty(JSON_PROPERTY_FEE_BALANCE_CHANGES)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public List<LtsFeeFungibleResourceBalanceChange> getFeeBalanceChanges() {
    return feeBalanceChanges;
  }


  @JsonProperty(JSON_PROPERTY_FEE_BALANCE_CHANGES)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setFeeBalanceChanges(List<LtsFeeFungibleResourceBalanceChange> feeBalanceChanges) {
    this.feeBalanceChanges = feeBalanceChanges;
  }


  public LtsEntityFungibleBalanceChanges nonFeeBalanceChanges(List<LtsFungibleResourceBalanceChange> nonFeeBalanceChanges) {
    this.nonFeeBalanceChanges = nonFeeBalanceChanges;
    return this;
  }

  public LtsEntityFungibleBalanceChanges addNonFeeBalanceChangesItem(LtsFungibleResourceBalanceChange nonFeeBalanceChangesItem) {
    this.nonFeeBalanceChanges.add(nonFeeBalanceChangesItem);
    return this;
  }

   /**
   * Get nonFeeBalanceChanges
   * @return nonFeeBalanceChanges
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_NON_FEE_BALANCE_CHANGES)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public List<LtsFungibleResourceBalanceChange> getNonFeeBalanceChanges() {
    return nonFeeBalanceChanges;
  }


  @JsonProperty(JSON_PROPERTY_NON_FEE_BALANCE_CHANGES)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setNonFeeBalanceChanges(List<LtsFungibleResourceBalanceChange> nonFeeBalanceChanges) {
    this.nonFeeBalanceChanges = nonFeeBalanceChanges;
  }


  /**
   * Return true if this LtsEntityFungibleBalanceChanges object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    LtsEntityFungibleBalanceChanges ltsEntityFungibleBalanceChanges = (LtsEntityFungibleBalanceChanges) o;
    return Objects.equals(this.entityAddress, ltsEntityFungibleBalanceChanges.entityAddress) &&
        Objects.equals(this.feeBalanceChange, ltsEntityFungibleBalanceChanges.feeBalanceChange) &&
        Objects.equals(this.feeBalanceChanges, ltsEntityFungibleBalanceChanges.feeBalanceChanges) &&
        Objects.equals(this.nonFeeBalanceChanges, ltsEntityFungibleBalanceChanges.nonFeeBalanceChanges);
  }

  @Override
  public int hashCode() {
    return Objects.hash(entityAddress, feeBalanceChange, feeBalanceChanges, nonFeeBalanceChanges);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class LtsEntityFungibleBalanceChanges {\n");
    sb.append("    entityAddress: ").append(toIndentedString(entityAddress)).append("\n");
    sb.append("    feeBalanceChange: ").append(toIndentedString(feeBalanceChange)).append("\n");
    sb.append("    feeBalanceChanges: ").append(toIndentedString(feeBalanceChanges)).append("\n");
    sb.append("    nonFeeBalanceChanges: ").append(toIndentedString(nonFeeBalanceChanges)).append("\n");
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

}

