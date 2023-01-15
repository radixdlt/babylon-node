/*
 * Babylon Core API
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node. It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Heavy load may impact the node's function.  If you require queries against historical ledger state, you may also wish to consider using the [Gateway API](https://betanet-gateway.redoc.ly/). 
 *
 * The version of the OpenAPI document: 0.1.0
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
import com.radixdlt.api.core.generated.models.EcdsaSecp256k1PublicKey;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import java.util.ArrayList;
import java.util.List;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * ValidatorSetSubstateAllOf
 */
@JsonPropertyOrder({
  ValidatorSetSubstateAllOf.JSON_PROPERTY_VALIDATOR_SET,
  ValidatorSetSubstateAllOf.JSON_PROPERTY_EPOCH
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class ValidatorSetSubstateAllOf {
  public static final String JSON_PROPERTY_VALIDATOR_SET = "validator_set";
  private List<EcdsaSecp256k1PublicKey> validatorSet = new ArrayList<>();

  public static final String JSON_PROPERTY_EPOCH = "epoch";
  private Long epoch;

  public ValidatorSetSubstateAllOf() { 
  }

  public ValidatorSetSubstateAllOf validatorSet(List<EcdsaSecp256k1PublicKey> validatorSet) {
    this.validatorSet = validatorSet;
    return this;
  }

  public ValidatorSetSubstateAllOf addValidatorSetItem(EcdsaSecp256k1PublicKey validatorSetItem) {
    this.validatorSet.add(validatorSetItem);
    return this;
  }

   /**
   * Get validatorSet
   * @return validatorSet
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_VALIDATOR_SET)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public List<EcdsaSecp256k1PublicKey> getValidatorSet() {
    return validatorSet;
  }


  @JsonProperty(JSON_PROPERTY_VALIDATOR_SET)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setValidatorSet(List<EcdsaSecp256k1PublicKey> validatorSet) {
    this.validatorSet = validatorSet;
  }


  public ValidatorSetSubstateAllOf epoch(Long epoch) {
    this.epoch = epoch;
    return this;
  }

   /**
   * An integer between &#x60;0&#x60; and &#x60;10^10&#x60;, marking the epoch the validator set is a part of
   * minimum: 0
   * maximum: 10000000000
   * @return epoch
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "An integer between `0` and `10^10`, marking the epoch the validator set is a part of")
  @JsonProperty(JSON_PROPERTY_EPOCH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Long getEpoch() {
    return epoch;
  }


  @JsonProperty(JSON_PROPERTY_EPOCH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setEpoch(Long epoch) {
    this.epoch = epoch;
  }


  /**
   * Return true if this ValidatorSetSubstate_allOf object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    ValidatorSetSubstateAllOf validatorSetSubstateAllOf = (ValidatorSetSubstateAllOf) o;
    return Objects.equals(this.validatorSet, validatorSetSubstateAllOf.validatorSet) &&
        Objects.equals(this.epoch, validatorSetSubstateAllOf.epoch);
  }

  @Override
  public int hashCode() {
    return Objects.hash(validatorSet, epoch);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class ValidatorSetSubstateAllOf {\n");
    sb.append("    validatorSet: ").append(toIndentedString(validatorSet)).append("\n");
    sb.append("    epoch: ").append(toIndentedString(epoch)).append("\n");
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

