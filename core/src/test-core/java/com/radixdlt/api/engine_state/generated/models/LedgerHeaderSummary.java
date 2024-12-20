/*
 * Engine State API (Beta)
 * **This API is currently in Beta**  This specification may experience breaking changes as part of Babylon Node releases. Such changes will be clearly mentioned in the [babylon-node release notes](https://github.com/radixdlt/babylon-node/releases). We advise against using this API for business-critical integrations before the `version` indicated above becomes stable, which is expected in Q4 of 2024.  This API provides a complete view of the current ledger state, operating at a relatively low level (i.e. returning Entities' data and type information in a generic way, without interpreting specifics of different native or custom components).  It mirrors how the Radix Engine views the ledger state in its \"System\" layer, and thus can be useful for Scrypto developers, who need to inspect how the Engine models and stores their application's state, or how an interface / authentication scheme of another component looks like. 
 *
 * The version of the OpenAPI document: v1.3.0
 * 
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */


package com.radixdlt.api.engine_state.generated.models;

import java.util.Objects;
import java.util.Arrays;
import java.util.Map;
import java.util.HashMap;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonTypeName;
import com.fasterxml.jackson.annotation.JsonValue;
import com.radixdlt.api.engine_state.generated.models.ConsensusInstant;
import com.radixdlt.api.engine_state.generated.models.EpochRound;
import com.radixdlt.api.engine_state.generated.models.LedgerHashes;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * LedgerHeaderSummary
 */
@JsonPropertyOrder({
  LedgerHeaderSummary.JSON_PROPERTY_EPOCH_ROUND,
  LedgerHeaderSummary.JSON_PROPERTY_LEDGER_HASHES,
  LedgerHeaderSummary.JSON_PROPERTY_PROPOSER_TIMESTAMP
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class LedgerHeaderSummary {
  public static final String JSON_PROPERTY_EPOCH_ROUND = "epoch_round";
  private EpochRound epochRound;

  public static final String JSON_PROPERTY_LEDGER_HASHES = "ledger_hashes";
  private LedgerHashes ledgerHashes;

  public static final String JSON_PROPERTY_PROPOSER_TIMESTAMP = "proposer_timestamp";
  private ConsensusInstant proposerTimestamp;

  public LedgerHeaderSummary() { 
  }

  public LedgerHeaderSummary epochRound(EpochRound epochRound) {
    this.epochRound = epochRound;
    return this;
  }

   /**
   * Get epochRound
   * @return epochRound
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_EPOCH_ROUND)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public EpochRound getEpochRound() {
    return epochRound;
  }


  @JsonProperty(JSON_PROPERTY_EPOCH_ROUND)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setEpochRound(EpochRound epochRound) {
    this.epochRound = epochRound;
  }


  public LedgerHeaderSummary ledgerHashes(LedgerHashes ledgerHashes) {
    this.ledgerHashes = ledgerHashes;
    return this;
  }

   /**
   * Get ledgerHashes
   * @return ledgerHashes
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_LEDGER_HASHES)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public LedgerHashes getLedgerHashes() {
    return ledgerHashes;
  }


  @JsonProperty(JSON_PROPERTY_LEDGER_HASHES)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setLedgerHashes(LedgerHashes ledgerHashes) {
    this.ledgerHashes = ledgerHashes;
  }


  public LedgerHeaderSummary proposerTimestamp(ConsensusInstant proposerTimestamp) {
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

  public ConsensusInstant getProposerTimestamp() {
    return proposerTimestamp;
  }


  @JsonProperty(JSON_PROPERTY_PROPOSER_TIMESTAMP)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setProposerTimestamp(ConsensusInstant proposerTimestamp) {
    this.proposerTimestamp = proposerTimestamp;
  }


  /**
   * Return true if this LedgerHeaderSummary object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    LedgerHeaderSummary ledgerHeaderSummary = (LedgerHeaderSummary) o;
    return Objects.equals(this.epochRound, ledgerHeaderSummary.epochRound) &&
        Objects.equals(this.ledgerHashes, ledgerHeaderSummary.ledgerHashes) &&
        Objects.equals(this.proposerTimestamp, ledgerHeaderSummary.proposerTimestamp);
  }

  @Override
  public int hashCode() {
    return Objects.hash(epochRound, ledgerHashes, proposerTimestamp);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class LedgerHeaderSummary {\n");
    sb.append("    epochRound: ").append(toIndentedString(epochRound)).append("\n");
    sb.append("    ledgerHashes: ").append(toIndentedString(ledgerHashes)).append("\n");
    sb.append("    proposerTimestamp: ").append(toIndentedString(proposerTimestamp)).append("\n");
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

