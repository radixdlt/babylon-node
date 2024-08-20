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
import com.radixdlt.api.core.generated.models.EpochRound;
import com.radixdlt.api.core.generated.models.InstantMs;
import com.radixdlt.api.core.generated.models.LedgerHashes;
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
  private InstantMs proposerTimestamp;

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


  public LedgerHeaderSummary proposerTimestamp(InstantMs proposerTimestamp) {
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

  public InstantMs getProposerTimestamp() {
    return proposerTimestamp;
  }


  @JsonProperty(JSON_PROPERTY_PROPOSER_TIMESTAMP)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setProposerTimestamp(InstantMs proposerTimestamp) {
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

