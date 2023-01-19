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
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * EpochManagerSubstateAllOf
 */
@JsonPropertyOrder({
  EpochManagerSubstateAllOf.JSON_PROPERTY_ADDRESS,
  EpochManagerSubstateAllOf.JSON_PROPERTY_EPOCH,
  EpochManagerSubstateAllOf.JSON_PROPERTY_ROUND,
  EpochManagerSubstateAllOf.JSON_PROPERTY_ROUNDS_PER_EPOCH
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class EpochManagerSubstateAllOf {
  public static final String JSON_PROPERTY_ADDRESS = "address";
  private String address;

  public static final String JSON_PROPERTY_EPOCH = "epoch";
  private Long epoch;

  public static final String JSON_PROPERTY_ROUND = "round";
  private Long round;

  public static final String JSON_PROPERTY_ROUNDS_PER_EPOCH = "rounds_per_epoch";
  private Long roundsPerEpoch;

  public EpochManagerSubstateAllOf() { 
  }

  public EpochManagerSubstateAllOf address(String address) {
    this.address = address;
    return this;
  }

   /**
   * The Bech32m-encoded human readable version of the component address
   * @return address
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The Bech32m-encoded human readable version of the component address")
  @JsonProperty(JSON_PROPERTY_ADDRESS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getAddress() {
    return address;
  }


  @JsonProperty(JSON_PROPERTY_ADDRESS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setAddress(String address) {
    this.address = address;
  }


  public EpochManagerSubstateAllOf epoch(Long epoch) {
    this.epoch = epoch;
    return this;
  }

   /**
   * An integer between &#x60;0&#x60; and &#x60;10^10&#x60;, marking the current epoch
   * minimum: 0
   * maximum: 10000000000
   * @return epoch
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "An integer between `0` and `10^10`, marking the current epoch")
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


  public EpochManagerSubstateAllOf round(Long round) {
    this.round = round;
    return this;
  }

   /**
   * An integer between &#x60;0&#x60; and &#x60;10^10&#x60;, marking the current round in an epoch
   * minimum: 0
   * maximum: 10000000000
   * @return round
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "An integer between `0` and `10^10`, marking the current round in an epoch")
  @JsonProperty(JSON_PROPERTY_ROUND)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Long getRound() {
    return round;
  }


  @JsonProperty(JSON_PROPERTY_ROUND)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setRound(Long round) {
    this.round = round;
  }


  public EpochManagerSubstateAllOf roundsPerEpoch(Long roundsPerEpoch) {
    this.roundsPerEpoch = roundsPerEpoch;
    return this;
  }

   /**
   * An integer between &#x60;0&#x60; and &#x60;10^10&#x60;, specifying the number of rounds per epoch
   * minimum: 0
   * maximum: 10000000000
   * @return roundsPerEpoch
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "An integer between `0` and `10^10`, specifying the number of rounds per epoch")
  @JsonProperty(JSON_PROPERTY_ROUNDS_PER_EPOCH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Long getRoundsPerEpoch() {
    return roundsPerEpoch;
  }


  @JsonProperty(JSON_PROPERTY_ROUNDS_PER_EPOCH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setRoundsPerEpoch(Long roundsPerEpoch) {
    this.roundsPerEpoch = roundsPerEpoch;
  }


  /**
   * Return true if this EpochManagerSubstate_allOf object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    EpochManagerSubstateAllOf epochManagerSubstateAllOf = (EpochManagerSubstateAllOf) o;
    return Objects.equals(this.address, epochManagerSubstateAllOf.address) &&
        Objects.equals(this.epoch, epochManagerSubstateAllOf.epoch) &&
        Objects.equals(this.round, epochManagerSubstateAllOf.round) &&
        Objects.equals(this.roundsPerEpoch, epochManagerSubstateAllOf.roundsPerEpoch);
  }

  @Override
  public int hashCode() {
    return Objects.hash(address, epoch, round, roundsPerEpoch);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class EpochManagerSubstateAllOf {\n");
    sb.append("    address: ").append(toIndentedString(address)).append("\n");
    sb.append("    epoch: ").append(toIndentedString(epoch)).append("\n");
    sb.append("    round: ").append(toIndentedString(round)).append("\n");
    sb.append("    roundsPerEpoch: ").append(toIndentedString(roundsPerEpoch)).append("\n");
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

