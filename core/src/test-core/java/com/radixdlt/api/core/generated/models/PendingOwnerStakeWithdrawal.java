/*
 * Radix Core API
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.2.3
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
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * PendingOwnerStakeWithdrawal
 */
@JsonPropertyOrder({
  PendingOwnerStakeWithdrawal.JSON_PROPERTY_EPOCH_UNLOCKED,
  PendingOwnerStakeWithdrawal.JSON_PROPERTY_STAKE_UNIT_AMOUNT
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class PendingOwnerStakeWithdrawal {
  public static final String JSON_PROPERTY_EPOCH_UNLOCKED = "epoch_unlocked";
  private Long epochUnlocked;

  public static final String JSON_PROPERTY_STAKE_UNIT_AMOUNT = "stake_unit_amount";
  private String stakeUnitAmount;

  public PendingOwnerStakeWithdrawal() { 
  }

  public PendingOwnerStakeWithdrawal epochUnlocked(Long epochUnlocked) {
    this.epochUnlocked = epochUnlocked;
    return this;
  }

   /**
   * An integer between &#x60;0&#x60; and &#x60;10^10&#x60;, marking the epoch when the stake units are unlocked for withdrawal. 
   * minimum: 0
   * maximum: 10000000000
   * @return epochUnlocked
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "An integer between `0` and `10^10`, marking the epoch when the stake units are unlocked for withdrawal. ")
  @JsonProperty(JSON_PROPERTY_EPOCH_UNLOCKED)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Long getEpochUnlocked() {
    return epochUnlocked;
  }


  @JsonProperty(JSON_PROPERTY_EPOCH_UNLOCKED)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setEpochUnlocked(Long epochUnlocked) {
    this.epochUnlocked = epochUnlocked;
  }


  public PendingOwnerStakeWithdrawal stakeUnitAmount(String stakeUnitAmount) {
    this.stakeUnitAmount = stakeUnitAmount;
    return this;
  }

   /**
   * A string-encoded fixed-precision decimal to 18 decimal places. A decimal is formed of some signed integer &#x60;m&#x60; of attos (&#x60;10^(-18)&#x60;) units, where &#x60;-2^(192 - 1) &lt;&#x3D; m &lt; 2^(192 - 1)&#x60;. 
   * @return stakeUnitAmount
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "A string-encoded fixed-precision decimal to 18 decimal places. A decimal is formed of some signed integer `m` of attos (`10^(-18)`) units, where `-2^(192 - 1) <= m < 2^(192 - 1)`. ")
  @JsonProperty(JSON_PROPERTY_STAKE_UNIT_AMOUNT)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getStakeUnitAmount() {
    return stakeUnitAmount;
  }


  @JsonProperty(JSON_PROPERTY_STAKE_UNIT_AMOUNT)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setStakeUnitAmount(String stakeUnitAmount) {
    this.stakeUnitAmount = stakeUnitAmount;
  }


  /**
   * Return true if this PendingOwnerStakeWithdrawal object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    PendingOwnerStakeWithdrawal pendingOwnerStakeWithdrawal = (PendingOwnerStakeWithdrawal) o;
    return Objects.equals(this.epochUnlocked, pendingOwnerStakeWithdrawal.epochUnlocked) &&
        Objects.equals(this.stakeUnitAmount, pendingOwnerStakeWithdrawal.stakeUnitAmount);
  }

  @Override
  public int hashCode() {
    return Objects.hash(epochUnlocked, stakeUnitAmount);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class PendingOwnerStakeWithdrawal {\n");
    sb.append("    epochUnlocked: ").append(toIndentedString(epochUnlocked)).append("\n");
    sb.append("    stakeUnitAmount: ").append(toIndentedString(stakeUnitAmount)).append("\n");
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

