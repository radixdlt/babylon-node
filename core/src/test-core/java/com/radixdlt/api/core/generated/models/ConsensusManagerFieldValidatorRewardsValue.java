/*
 * Babylon Core API - RCnet v3.1
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  This version of the Core API belongs to the fourth release candidate of the Radix Babylon network (\"RCnet v3.1\").  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` are guaranteed to be forward compatible to Babylon mainnet launch (and beyond). We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code. 
 *
 * The version of the OpenAPI document: 0.5.0
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
import com.radixdlt.api.core.generated.models.EntityReference;
import com.radixdlt.api.core.generated.models.ProposerReward;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import java.util.ArrayList;
import java.util.List;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * ConsensusManagerFieldValidatorRewardsValue
 */
@JsonPropertyOrder({
  ConsensusManagerFieldValidatorRewardsValue.JSON_PROPERTY_PROPOSER_REWARDS,
  ConsensusManagerFieldValidatorRewardsValue.JSON_PROPERTY_REWARDS_VAULT
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class ConsensusManagerFieldValidatorRewardsValue {
  public static final String JSON_PROPERTY_PROPOSER_REWARDS = "proposer_rewards";
  private List<ProposerReward> proposerRewards = new ArrayList<>();

  public static final String JSON_PROPERTY_REWARDS_VAULT = "rewards_vault";
  private EntityReference rewardsVault;

  public ConsensusManagerFieldValidatorRewardsValue() { 
  }

  public ConsensusManagerFieldValidatorRewardsValue proposerRewards(List<ProposerReward> proposerRewards) {
    this.proposerRewards = proposerRewards;
    return this;
  }

  public ConsensusManagerFieldValidatorRewardsValue addProposerRewardsItem(ProposerReward proposerRewardsItem) {
    this.proposerRewards.add(proposerRewardsItem);
    return this;
  }

   /**
   * Get proposerRewards
   * @return proposerRewards
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_PROPOSER_REWARDS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public List<ProposerReward> getProposerRewards() {
    return proposerRewards;
  }


  @JsonProperty(JSON_PROPERTY_PROPOSER_REWARDS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setProposerRewards(List<ProposerReward> proposerRewards) {
    this.proposerRewards = proposerRewards;
  }


  public ConsensusManagerFieldValidatorRewardsValue rewardsVault(EntityReference rewardsVault) {
    this.rewardsVault = rewardsVault;
    return this;
  }

   /**
   * Get rewardsVault
   * @return rewardsVault
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_REWARDS_VAULT)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public EntityReference getRewardsVault() {
    return rewardsVault;
  }


  @JsonProperty(JSON_PROPERTY_REWARDS_VAULT)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setRewardsVault(EntityReference rewardsVault) {
    this.rewardsVault = rewardsVault;
  }


  /**
   * Return true if this ConsensusManagerFieldValidatorRewardsValue object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    ConsensusManagerFieldValidatorRewardsValue consensusManagerFieldValidatorRewardsValue = (ConsensusManagerFieldValidatorRewardsValue) o;
    return Objects.equals(this.proposerRewards, consensusManagerFieldValidatorRewardsValue.proposerRewards) &&
        Objects.equals(this.rewardsVault, consensusManagerFieldValidatorRewardsValue.rewardsVault);
  }

  @Override
  public int hashCode() {
    return Objects.hash(proposerRewards, rewardsVault);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class ConsensusManagerFieldValidatorRewardsValue {\n");
    sb.append("    proposerRewards: ").append(toIndentedString(proposerRewards)).append("\n");
    sb.append("    rewardsVault: ").append(toIndentedString(rewardsVault)).append("\n");
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

