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
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * ActiveValidatorKey
 */
@JsonPropertyOrder({
  ActiveValidatorKey.JSON_PROPERTY_STAKE_WEIGHTING,
  ActiveValidatorKey.JSON_PROPERTY_VALIDATOR_ADDRESS
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class ActiveValidatorKey {
  public static final String JSON_PROPERTY_STAKE_WEIGHTING = "stake_weighting";
  private Integer stakeWeighting;

  public static final String JSON_PROPERTY_VALIDATOR_ADDRESS = "validator_address";
  private String validatorAddress;

  public ActiveValidatorKey() { 
  }

  public ActiveValidatorKey stakeWeighting(Integer stakeWeighting) {
    this.stakeWeighting = stakeWeighting;
    return this;
  }

   /**
   * The divided stake, giving a weighting for the validator, used as part of the sort key. 
   * minimum: 0
   * maximum: 65535
   * @return stakeWeighting
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The divided stake, giving a weighting for the validator, used as part of the sort key. ")
  @JsonProperty(JSON_PROPERTY_STAKE_WEIGHTING)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Integer getStakeWeighting() {
    return stakeWeighting;
  }


  @JsonProperty(JSON_PROPERTY_STAKE_WEIGHTING)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setStakeWeighting(Integer stakeWeighting) {
    this.stakeWeighting = stakeWeighting;
  }


  public ActiveValidatorKey validatorAddress(String validatorAddress) {
    this.validatorAddress = validatorAddress;
    return this;
  }

   /**
   * The Bech32m-encoded human readable version of the component address
   * @return validatorAddress
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The Bech32m-encoded human readable version of the component address")
  @JsonProperty(JSON_PROPERTY_VALIDATOR_ADDRESS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getValidatorAddress() {
    return validatorAddress;
  }


  @JsonProperty(JSON_PROPERTY_VALIDATOR_ADDRESS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setValidatorAddress(String validatorAddress) {
    this.validatorAddress = validatorAddress;
  }


  /**
   * Return true if this ActiveValidatorKey object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    ActiveValidatorKey activeValidatorKey = (ActiveValidatorKey) o;
    return Objects.equals(this.stakeWeighting, activeValidatorKey.stakeWeighting) &&
        Objects.equals(this.validatorAddress, activeValidatorKey.validatorAddress);
  }

  @Override
  public int hashCode() {
    return Objects.hash(stakeWeighting, validatorAddress);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class ActiveValidatorKey {\n");
    sb.append("    stakeWeighting: ").append(toIndentedString(stakeWeighting)).append("\n");
    sb.append("    validatorAddress: ").append(toIndentedString(validatorAddress)).append("\n");
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

