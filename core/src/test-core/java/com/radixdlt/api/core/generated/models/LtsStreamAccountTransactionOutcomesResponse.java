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
import com.radixdlt.api.core.generated.models.LtsCommittedTransactionOutcome;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import java.util.ArrayList;
import java.util.List;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * LtsStreamAccountTransactionOutcomesResponse
 */
@JsonPropertyOrder({
  LtsStreamAccountTransactionOutcomesResponse.JSON_PROPERTY_FROM_STATE_VERSION,
  LtsStreamAccountTransactionOutcomesResponse.JSON_PROPERTY_COUNT,
  LtsStreamAccountTransactionOutcomesResponse.JSON_PROPERTY_MAX_LEDGER_STATE_VERSION,
  LtsStreamAccountTransactionOutcomesResponse.JSON_PROPERTY_COMMITTED_TRANSACTION_OUTCOMES
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class LtsStreamAccountTransactionOutcomesResponse {
  public static final String JSON_PROPERTY_FROM_STATE_VERSION = "from_state_version";
  private Long fromStateVersion;

  public static final String JSON_PROPERTY_COUNT = "count";
  private Integer count;

  public static final String JSON_PROPERTY_MAX_LEDGER_STATE_VERSION = "max_ledger_state_version";
  private Long maxLedgerStateVersion;

  public static final String JSON_PROPERTY_COMMITTED_TRANSACTION_OUTCOMES = "committed_transaction_outcomes";
  private List<LtsCommittedTransactionOutcome> committedTransactionOutcomes = new ArrayList<>();

  public LtsStreamAccountTransactionOutcomesResponse() { 
  }

  public LtsStreamAccountTransactionOutcomesResponse fromStateVersion(Long fromStateVersion) {
    this.fromStateVersion = fromStateVersion;
    return this;
  }

   /**
   * Get fromStateVersion
   * minimum: 1
   * maximum: 100000000000000
   * @return fromStateVersion
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_FROM_STATE_VERSION)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Long getFromStateVersion() {
    return fromStateVersion;
  }


  @JsonProperty(JSON_PROPERTY_FROM_STATE_VERSION)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setFromStateVersion(Long fromStateVersion) {
    this.fromStateVersion = fromStateVersion;
  }


  public LtsStreamAccountTransactionOutcomesResponse count(Integer count) {
    this.count = count;
    return this;
  }

   /**
   * An integer between &#x60;0&#x60; and &#x60;10000&#x60;, giving the total count of transactions in the returned response
   * minimum: 0
   * maximum: 10000
   * @return count
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "An integer between `0` and `10000`, giving the total count of transactions in the returned response")
  @JsonProperty(JSON_PROPERTY_COUNT)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Integer getCount() {
    return count;
  }


  @JsonProperty(JSON_PROPERTY_COUNT)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setCount(Integer count) {
    this.count = count;
  }


  public LtsStreamAccountTransactionOutcomesResponse maxLedgerStateVersion(Long maxLedgerStateVersion) {
    this.maxLedgerStateVersion = maxLedgerStateVersion;
    return this;
  }

   /**
   * Get maxLedgerStateVersion
   * minimum: 1
   * maximum: 100000000000000
   * @return maxLedgerStateVersion
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_MAX_LEDGER_STATE_VERSION)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Long getMaxLedgerStateVersion() {
    return maxLedgerStateVersion;
  }


  @JsonProperty(JSON_PROPERTY_MAX_LEDGER_STATE_VERSION)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setMaxLedgerStateVersion(Long maxLedgerStateVersion) {
    this.maxLedgerStateVersion = maxLedgerStateVersion;
  }


  public LtsStreamAccountTransactionOutcomesResponse committedTransactionOutcomes(List<LtsCommittedTransactionOutcome> committedTransactionOutcomes) {
    this.committedTransactionOutcomes = committedTransactionOutcomes;
    return this;
  }

  public LtsStreamAccountTransactionOutcomesResponse addCommittedTransactionOutcomesItem(LtsCommittedTransactionOutcome committedTransactionOutcomesItem) {
    this.committedTransactionOutcomes.add(committedTransactionOutcomesItem);
    return this;
  }

   /**
   * A committed transaction outcomes list starting from the &#x60;from_state_version&#x60; (inclusive).
   * @return committedTransactionOutcomes
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "A committed transaction outcomes list starting from the `from_state_version` (inclusive).")
  @JsonProperty(JSON_PROPERTY_COMMITTED_TRANSACTION_OUTCOMES)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public List<LtsCommittedTransactionOutcome> getCommittedTransactionOutcomes() {
    return committedTransactionOutcomes;
  }


  @JsonProperty(JSON_PROPERTY_COMMITTED_TRANSACTION_OUTCOMES)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setCommittedTransactionOutcomes(List<LtsCommittedTransactionOutcome> committedTransactionOutcomes) {
    this.committedTransactionOutcomes = committedTransactionOutcomes;
  }


  /**
   * Return true if this LtsStreamAccountTransactionOutcomesResponse object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    LtsStreamAccountTransactionOutcomesResponse ltsStreamAccountTransactionOutcomesResponse = (LtsStreamAccountTransactionOutcomesResponse) o;
    return Objects.equals(this.fromStateVersion, ltsStreamAccountTransactionOutcomesResponse.fromStateVersion) &&
        Objects.equals(this.count, ltsStreamAccountTransactionOutcomesResponse.count) &&
        Objects.equals(this.maxLedgerStateVersion, ltsStreamAccountTransactionOutcomesResponse.maxLedgerStateVersion) &&
        Objects.equals(this.committedTransactionOutcomes, ltsStreamAccountTransactionOutcomesResponse.committedTransactionOutcomes);
  }

  @Override
  public int hashCode() {
    return Objects.hash(fromStateVersion, count, maxLedgerStateVersion, committedTransactionOutcomes);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class LtsStreamAccountTransactionOutcomesResponse {\n");
    sb.append("    fromStateVersion: ").append(toIndentedString(fromStateVersion)).append("\n");
    sb.append("    count: ").append(toIndentedString(count)).append("\n");
    sb.append("    maxLedgerStateVersion: ").append(toIndentedString(maxLedgerStateVersion)).append("\n");
    sb.append("    committedTransactionOutcomes: ").append(toIndentedString(committedTransactionOutcomes)).append("\n");
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

