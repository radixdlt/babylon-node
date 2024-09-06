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
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonTypeName;
import com.fasterxml.jackson.annotation.JsonValue;
import com.radixdlt.api.core.generated.models.ActiveValidatorIndex;
import com.radixdlt.api.core.generated.models.InstantMs;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * ConsensusManagerFieldStateValue
 */
@JsonPropertyOrder({
  ConsensusManagerFieldStateValue.JSON_PROPERTY_EPOCH,
  ConsensusManagerFieldStateValue.JSON_PROPERTY_ROUND,
  ConsensusManagerFieldStateValue.JSON_PROPERTY_IS_STARTED,
  ConsensusManagerFieldStateValue.JSON_PROPERTY_EFFECTIVE_EPOCH_START,
  ConsensusManagerFieldStateValue.JSON_PROPERTY_ACTUAL_EPOCH_START,
  ConsensusManagerFieldStateValue.JSON_PROPERTY_CURRENT_LEADER
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class ConsensusManagerFieldStateValue {
  public static final String JSON_PROPERTY_EPOCH = "epoch";
  private Long epoch;

  public static final String JSON_PROPERTY_ROUND = "round";
  private Long round;

  public static final String JSON_PROPERTY_IS_STARTED = "is_started";
  private Boolean isStarted;

  public static final String JSON_PROPERTY_EFFECTIVE_EPOCH_START = "effective_epoch_start";
  private InstantMs effectiveEpochStart;

  public static final String JSON_PROPERTY_ACTUAL_EPOCH_START = "actual_epoch_start";
  private InstantMs actualEpochStart;

  public static final String JSON_PROPERTY_CURRENT_LEADER = "current_leader";
  private ActiveValidatorIndex currentLeader;

  public ConsensusManagerFieldStateValue() { 
  }

  public ConsensusManagerFieldStateValue epoch(Long epoch) {
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


  public ConsensusManagerFieldStateValue round(Long round) {
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


  public ConsensusManagerFieldStateValue isStarted(Boolean isStarted) {
    this.isStarted = isStarted;
    return this;
  }

   /**
   * Get isStarted
   * @return isStarted
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_IS_STARTED)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Boolean getIsStarted() {
    return isStarted;
  }


  @JsonProperty(JSON_PROPERTY_IS_STARTED)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setIsStarted(Boolean isStarted) {
    this.isStarted = isStarted;
  }


  public ConsensusManagerFieldStateValue effectiveEpochStart(InstantMs effectiveEpochStart) {
    this.effectiveEpochStart = effectiveEpochStart;
    return this;
  }

   /**
   * Get effectiveEpochStart
   * @return effectiveEpochStart
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_EFFECTIVE_EPOCH_START)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public InstantMs getEffectiveEpochStart() {
    return effectiveEpochStart;
  }


  @JsonProperty(JSON_PROPERTY_EFFECTIVE_EPOCH_START)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setEffectiveEpochStart(InstantMs effectiveEpochStart) {
    this.effectiveEpochStart = effectiveEpochStart;
  }


  public ConsensusManagerFieldStateValue actualEpochStart(InstantMs actualEpochStart) {
    this.actualEpochStart = actualEpochStart;
    return this;
  }

   /**
   * Get actualEpochStart
   * @return actualEpochStart
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_ACTUAL_EPOCH_START)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public InstantMs getActualEpochStart() {
    return actualEpochStart;
  }


  @JsonProperty(JSON_PROPERTY_ACTUAL_EPOCH_START)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setActualEpochStart(InstantMs actualEpochStart) {
    this.actualEpochStart = actualEpochStart;
  }


  public ConsensusManagerFieldStateValue currentLeader(ActiveValidatorIndex currentLeader) {
    this.currentLeader = currentLeader;
    return this;
  }

   /**
   * Get currentLeader
   * @return currentLeader
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "")
  @JsonProperty(JSON_PROPERTY_CURRENT_LEADER)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public ActiveValidatorIndex getCurrentLeader() {
    return currentLeader;
  }


  @JsonProperty(JSON_PROPERTY_CURRENT_LEADER)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setCurrentLeader(ActiveValidatorIndex currentLeader) {
    this.currentLeader = currentLeader;
  }


  /**
   * Return true if this ConsensusManagerFieldStateValue object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    ConsensusManagerFieldStateValue consensusManagerFieldStateValue = (ConsensusManagerFieldStateValue) o;
    return Objects.equals(this.epoch, consensusManagerFieldStateValue.epoch) &&
        Objects.equals(this.round, consensusManagerFieldStateValue.round) &&
        Objects.equals(this.isStarted, consensusManagerFieldStateValue.isStarted) &&
        Objects.equals(this.effectiveEpochStart, consensusManagerFieldStateValue.effectiveEpochStart) &&
        Objects.equals(this.actualEpochStart, consensusManagerFieldStateValue.actualEpochStart) &&
        Objects.equals(this.currentLeader, consensusManagerFieldStateValue.currentLeader);
  }

  @Override
  public int hashCode() {
    return Objects.hash(epoch, round, isStarted, effectiveEpochStart, actualEpochStart, currentLeader);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class ConsensusManagerFieldStateValue {\n");
    sb.append("    epoch: ").append(toIndentedString(epoch)).append("\n");
    sb.append("    round: ").append(toIndentedString(round)).append("\n");
    sb.append("    isStarted: ").append(toIndentedString(isStarted)).append("\n");
    sb.append("    effectiveEpochStart: ").append(toIndentedString(effectiveEpochStart)).append("\n");
    sb.append("    actualEpochStart: ").append(toIndentedString(actualEpochStart)).append("\n");
    sb.append("    currentLeader: ").append(toIndentedString(currentLeader)).append("\n");
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

