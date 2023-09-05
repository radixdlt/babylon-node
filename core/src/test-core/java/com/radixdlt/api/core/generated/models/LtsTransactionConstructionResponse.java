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
import com.radixdlt.api.core.generated.models.Instant;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * LtsTransactionConstructionResponse
 */
@JsonPropertyOrder({
  LtsTransactionConstructionResponse.JSON_PROPERTY_CURRENT_EPOCH,
  LtsTransactionConstructionResponse.JSON_PROPERTY_LEDGER_CLOCK
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class LtsTransactionConstructionResponse {
  public static final String JSON_PROPERTY_CURRENT_EPOCH = "current_epoch";
  private Long currentEpoch;

  public static final String JSON_PROPERTY_LEDGER_CLOCK = "ledger_clock";
  private Instant ledgerClock;

  public LtsTransactionConstructionResponse() { 
  }

  public LtsTransactionConstructionResponse currentEpoch(Long currentEpoch) {
    this.currentEpoch = currentEpoch;
    return this;
  }

   /**
   * An integer between &#x60;0&#x60; and &#x60;10^10&#x60;, marking the current epoch
   * minimum: 0
   * maximum: 10000000000
   * @return currentEpoch
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "An integer between `0` and `10^10`, marking the current epoch")
  @JsonProperty(JSON_PROPERTY_CURRENT_EPOCH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Long getCurrentEpoch() {
    return currentEpoch;
  }


  @JsonProperty(JSON_PROPERTY_CURRENT_EPOCH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setCurrentEpoch(Long currentEpoch) {
    this.currentEpoch = currentEpoch;
  }


  public LtsTransactionConstructionResponse ledgerClock(Instant ledgerClock) {
    this.ledgerClock = ledgerClock;
    return this;
  }

   /**
   * Get ledgerClock
   * @return ledgerClock
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_LEDGER_CLOCK)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Instant getLedgerClock() {
    return ledgerClock;
  }


  @JsonProperty(JSON_PROPERTY_LEDGER_CLOCK)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setLedgerClock(Instant ledgerClock) {
    this.ledgerClock = ledgerClock;
  }


  /**
   * Return true if this LtsTransactionConstructionResponse object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    LtsTransactionConstructionResponse ltsTransactionConstructionResponse = (LtsTransactionConstructionResponse) o;
    return Objects.equals(this.currentEpoch, ltsTransactionConstructionResponse.currentEpoch) &&
        Objects.equals(this.ledgerClock, ltsTransactionConstructionResponse.ledgerClock);
  }

  @Override
  public int hashCode() {
    return Objects.hash(currentEpoch, ledgerClock);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class LtsTransactionConstructionResponse {\n");
    sb.append("    currentEpoch: ").append(toIndentedString(currentEpoch)).append("\n");
    sb.append("    ledgerClock: ").append(toIndentedString(ledgerClock)).append("\n");
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

