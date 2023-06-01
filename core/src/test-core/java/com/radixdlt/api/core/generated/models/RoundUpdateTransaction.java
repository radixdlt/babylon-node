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
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonTypeName;
import com.fasterxml.jackson.annotation.JsonValue;
import com.radixdlt.api.core.generated.models.Instant;
import com.radixdlt.api.core.generated.models.LeaderProposalHistory;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * RoundUpdateTransaction
 */
@JsonPropertyOrder({
  RoundUpdateTransaction.JSON_PROPERTY_PROPOSER_TIMESTAMP,
  RoundUpdateTransaction.JSON_PROPERTY_EPOCH,
  RoundUpdateTransaction.JSON_PROPERTY_ROUND_IN_EPOCH,
  RoundUpdateTransaction.JSON_PROPERTY_LEADER_PROPOSAL_HISTORY
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class RoundUpdateTransaction {
  public static final String JSON_PROPERTY_PROPOSER_TIMESTAMP = "proposer_timestamp";
  private Instant proposerTimestamp;

  public static final String JSON_PROPERTY_EPOCH = "epoch";
  private Long epoch;

  public static final String JSON_PROPERTY_ROUND_IN_EPOCH = "round_in_epoch";
  private Long roundInEpoch;

  public static final String JSON_PROPERTY_LEADER_PROPOSAL_HISTORY = "leader_proposal_history";
  private LeaderProposalHistory leaderProposalHistory;

  public RoundUpdateTransaction() { 
  }

  public RoundUpdateTransaction proposerTimestamp(Instant proposerTimestamp) {
    this.proposerTimestamp = proposerTimestamp;
    return this;
  }

   /**
   * Get proposerTimestamp
   * @return proposerTimestamp
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_PROPOSER_TIMESTAMP)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Instant getProposerTimestamp() {
    return proposerTimestamp;
  }


  @JsonProperty(JSON_PROPERTY_PROPOSER_TIMESTAMP)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setProposerTimestamp(Instant proposerTimestamp) {
    this.proposerTimestamp = proposerTimestamp;
  }


  public RoundUpdateTransaction epoch(Long epoch) {
    this.epoch = epoch;
    return this;
  }

   /**
   * An integer between &#x60;0&#x60; and &#x60;10^10&#x60;, marking the epoch. 
   * minimum: 0
   * maximum: 10000000000
   * @return epoch
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "An integer between `0` and `10^10`, marking the epoch. ")
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


  public RoundUpdateTransaction roundInEpoch(Long roundInEpoch) {
    this.roundInEpoch = roundInEpoch;
    return this;
  }

   /**
   * An integer between &#x60;0&#x60; and &#x60;10^10&#x60;, marking the consensus round in the epoch
   * minimum: 0
   * maximum: 10000000000
   * @return roundInEpoch
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "An integer between `0` and `10^10`, marking the consensus round in the epoch")
  @JsonProperty(JSON_PROPERTY_ROUND_IN_EPOCH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Long getRoundInEpoch() {
    return roundInEpoch;
  }


  @JsonProperty(JSON_PROPERTY_ROUND_IN_EPOCH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setRoundInEpoch(Long roundInEpoch) {
    this.roundInEpoch = roundInEpoch;
  }


  public RoundUpdateTransaction leaderProposalHistory(LeaderProposalHistory leaderProposalHistory) {
    this.leaderProposalHistory = leaderProposalHistory;
    return this;
  }

   /**
   * Get leaderProposalHistory
   * @return leaderProposalHistory
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_LEADER_PROPOSAL_HISTORY)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public LeaderProposalHistory getLeaderProposalHistory() {
    return leaderProposalHistory;
  }


  @JsonProperty(JSON_PROPERTY_LEADER_PROPOSAL_HISTORY)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setLeaderProposalHistory(LeaderProposalHistory leaderProposalHistory) {
    this.leaderProposalHistory = leaderProposalHistory;
  }


  /**
   * Return true if this RoundUpdateTransaction object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    RoundUpdateTransaction roundUpdateTransaction = (RoundUpdateTransaction) o;
    return Objects.equals(this.proposerTimestamp, roundUpdateTransaction.proposerTimestamp) &&
        Objects.equals(this.epoch, roundUpdateTransaction.epoch) &&
        Objects.equals(this.roundInEpoch, roundUpdateTransaction.roundInEpoch) &&
        Objects.equals(this.leaderProposalHistory, roundUpdateTransaction.leaderProposalHistory);
  }

  @Override
  public int hashCode() {
    return Objects.hash(proposerTimestamp, epoch, roundInEpoch, leaderProposalHistory);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class RoundUpdateTransaction {\n");
    sb.append("    proposerTimestamp: ").append(toIndentedString(proposerTimestamp)).append("\n");
    sb.append("    epoch: ").append(toIndentedString(epoch)).append("\n");
    sb.append("    roundInEpoch: ").append(toIndentedString(roundInEpoch)).append("\n");
    sb.append("    leaderProposalHistory: ").append(toIndentedString(leaderProposalHistory)).append("\n");
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
