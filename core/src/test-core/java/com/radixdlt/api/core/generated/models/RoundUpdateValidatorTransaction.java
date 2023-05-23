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
import com.radixdlt.api.core.generated.models.Instant;
import com.radixdlt.api.core.generated.models.LeaderProposalHistory;
import com.radixdlt.api.core.generated.models.RoundUpdateValidatorTransaction;
import com.radixdlt.api.core.generated.models.RoundUpdateValidatorTransactionAllOf;
import com.radixdlt.api.core.generated.models.ValidatorTransaction;
import com.radixdlt.api.core.generated.models.ValidatorTransactionType;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


import com.radixdlt.api.core.generated.client.JSON;
/**
 * RoundUpdateValidatorTransaction
 */
@JsonPropertyOrder({
  RoundUpdateValidatorTransaction.JSON_PROPERTY_PROPOSER_TIMESTAMP,
  RoundUpdateValidatorTransaction.JSON_PROPERTY_CONSENSUS_EPOCH,
  RoundUpdateValidatorTransaction.JSON_PROPERTY_ROUND_IN_EPOCH,
  RoundUpdateValidatorTransaction.JSON_PROPERTY_LEADER_PROPOSAL_HISTORY
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
@JsonIgnoreProperties(
  value = "type", // ignore manually set type, it will be automatically generated by Jackson during serialization
  allowSetters = true // allows the type to be set during deserialization
)
@JsonTypeInfo(use = JsonTypeInfo.Id.NAME, include = JsonTypeInfo.As.PROPERTY, property = "type", visible = true)
@JsonSubTypes({
  @JsonSubTypes.Type(value = RoundUpdateValidatorTransaction.class, name = "RoundUpdate"),
})

public class RoundUpdateValidatorTransaction extends ValidatorTransaction {
  public static final String JSON_PROPERTY_PROPOSER_TIMESTAMP = "proposer_timestamp";
  private Instant proposerTimestamp;

  public static final String JSON_PROPERTY_CONSENSUS_EPOCH = "consensus_epoch";
  private Long consensusEpoch;

  public static final String JSON_PROPERTY_ROUND_IN_EPOCH = "round_in_epoch";
  private Long roundInEpoch;

  public static final String JSON_PROPERTY_LEADER_PROPOSAL_HISTORY = "leader_proposal_history";
  private LeaderProposalHistory leaderProposalHistory;

  public RoundUpdateValidatorTransaction() { 
  }

  public RoundUpdateValidatorTransaction proposerTimestamp(Instant proposerTimestamp) {
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


  public RoundUpdateValidatorTransaction consensusEpoch(Long consensusEpoch) {
    this.consensusEpoch = consensusEpoch;
    return this;
  }

   /**
   * An integer between &#x60;0&#x60; and &#x60;10^10&#x60;, marking the consensus epoch. 
   * minimum: 0
   * maximum: 10000000000
   * @return consensusEpoch
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "An integer between `0` and `10^10`, marking the consensus epoch. ")
  @JsonProperty(JSON_PROPERTY_CONSENSUS_EPOCH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Long getConsensusEpoch() {
    return consensusEpoch;
  }


  @JsonProperty(JSON_PROPERTY_CONSENSUS_EPOCH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setConsensusEpoch(Long consensusEpoch) {
    this.consensusEpoch = consensusEpoch;
  }


  public RoundUpdateValidatorTransaction roundInEpoch(Long roundInEpoch) {
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


  public RoundUpdateValidatorTransaction leaderProposalHistory(LeaderProposalHistory leaderProposalHistory) {
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
   * Return true if this RoundUpdateValidatorTransaction object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    RoundUpdateValidatorTransaction roundUpdateValidatorTransaction = (RoundUpdateValidatorTransaction) o;
    return Objects.equals(this.proposerTimestamp, roundUpdateValidatorTransaction.proposerTimestamp) &&
        Objects.equals(this.consensusEpoch, roundUpdateValidatorTransaction.consensusEpoch) &&
        Objects.equals(this.roundInEpoch, roundUpdateValidatorTransaction.roundInEpoch) &&
        Objects.equals(this.leaderProposalHistory, roundUpdateValidatorTransaction.leaderProposalHistory) &&
        super.equals(o);
  }

  @Override
  public int hashCode() {
    return Objects.hash(proposerTimestamp, consensusEpoch, roundInEpoch, leaderProposalHistory, super.hashCode());
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class RoundUpdateValidatorTransaction {\n");
    sb.append("    ").append(toIndentedString(super.toString())).append("\n");
    sb.append("    proposerTimestamp: ").append(toIndentedString(proposerTimestamp)).append("\n");
    sb.append("    consensusEpoch: ").append(toIndentedString(consensusEpoch)).append("\n");
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

static {
  // Initialize and register the discriminator mappings.
  Map<String, Class<?>> mappings = new HashMap<String, Class<?>>();
  mappings.put("RoundUpdate", RoundUpdateValidatorTransaction.class);
  mappings.put("RoundUpdateValidatorTransaction", RoundUpdateValidatorTransaction.class);
  JSON.registerDiscriminator(RoundUpdateValidatorTransaction.class, "type", mappings);
}
}

