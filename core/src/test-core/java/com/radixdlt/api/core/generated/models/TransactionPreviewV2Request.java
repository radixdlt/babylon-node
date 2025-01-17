/*
 * Radix Core API
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.3.0
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
import com.radixdlt.api.core.generated.models.LedgerStateSelector;
import com.radixdlt.api.core.generated.models.PreviewFlags;
import com.radixdlt.api.core.generated.models.PreviewTransaction;
import com.radixdlt.api.core.generated.models.TransactionPreviewV2ResponseOptions;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * TransactionPreviewV2Request
 */
@JsonPropertyOrder({
  TransactionPreviewV2Request.JSON_PROPERTY_NETWORK,
  TransactionPreviewV2Request.JSON_PROPERTY_AT_LEDGER_STATE,
  TransactionPreviewV2Request.JSON_PROPERTY_PREVIEW_TRANSACTION,
  TransactionPreviewV2Request.JSON_PROPERTY_FLAGS,
  TransactionPreviewV2Request.JSON_PROPERTY_OPTIONS
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class TransactionPreviewV2Request {
  public static final String JSON_PROPERTY_NETWORK = "network";
  private String network;

  public static final String JSON_PROPERTY_AT_LEDGER_STATE = "at_ledger_state";
  private LedgerStateSelector atLedgerState;

  public static final String JSON_PROPERTY_PREVIEW_TRANSACTION = "preview_transaction";
  private PreviewTransaction previewTransaction;

  public static final String JSON_PROPERTY_FLAGS = "flags";
  private PreviewFlags flags;

  public static final String JSON_PROPERTY_OPTIONS = "options";
  private TransactionPreviewV2ResponseOptions options;

  public TransactionPreviewV2Request() { 
  }

  public TransactionPreviewV2Request network(String network) {
    this.network = network;
    return this;
  }

   /**
   * The logical name of the network
   * @return network
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(example = "{{network}}", required = true, value = "The logical name of the network")
  @JsonProperty(JSON_PROPERTY_NETWORK)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getNetwork() {
    return network;
  }


  @JsonProperty(JSON_PROPERTY_NETWORK)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setNetwork(String network) {
    this.network = network;
  }


  public TransactionPreviewV2Request atLedgerState(LedgerStateSelector atLedgerState) {
    this.atLedgerState = atLedgerState;
    return this;
  }

   /**
   * Get atLedgerState
   * @return atLedgerState
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "")
  @JsonProperty(JSON_PROPERTY_AT_LEDGER_STATE)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public LedgerStateSelector getAtLedgerState() {
    return atLedgerState;
  }


  @JsonProperty(JSON_PROPERTY_AT_LEDGER_STATE)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setAtLedgerState(LedgerStateSelector atLedgerState) {
    this.atLedgerState = atLedgerState;
  }


  public TransactionPreviewV2Request previewTransaction(PreviewTransaction previewTransaction) {
    this.previewTransaction = previewTransaction;
    return this;
  }

   /**
   * Get previewTransaction
   * @return previewTransaction
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_PREVIEW_TRANSACTION)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public PreviewTransaction getPreviewTransaction() {
    return previewTransaction;
  }


  @JsonProperty(JSON_PROPERTY_PREVIEW_TRANSACTION)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setPreviewTransaction(PreviewTransaction previewTransaction) {
    this.previewTransaction = previewTransaction;
  }


  public TransactionPreviewV2Request flags(PreviewFlags flags) {
    this.flags = flags;
    return this;
  }

   /**
   * Get flags
   * @return flags
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "")
  @JsonProperty(JSON_PROPERTY_FLAGS)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public PreviewFlags getFlags() {
    return flags;
  }


  @JsonProperty(JSON_PROPERTY_FLAGS)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setFlags(PreviewFlags flags) {
    this.flags = flags;
  }


  public TransactionPreviewV2Request options(TransactionPreviewV2ResponseOptions options) {
    this.options = options;
    return this;
  }

   /**
   * Get options
   * @return options
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "")
  @JsonProperty(JSON_PROPERTY_OPTIONS)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public TransactionPreviewV2ResponseOptions getOptions() {
    return options;
  }


  @JsonProperty(JSON_PROPERTY_OPTIONS)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setOptions(TransactionPreviewV2ResponseOptions options) {
    this.options = options;
  }


  /**
   * Return true if this TransactionPreviewV2Request object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    TransactionPreviewV2Request transactionPreviewV2Request = (TransactionPreviewV2Request) o;
    return Objects.equals(this.network, transactionPreviewV2Request.network) &&
        Objects.equals(this.atLedgerState, transactionPreviewV2Request.atLedgerState) &&
        Objects.equals(this.previewTransaction, transactionPreviewV2Request.previewTransaction) &&
        Objects.equals(this.flags, transactionPreviewV2Request.flags) &&
        Objects.equals(this.options, transactionPreviewV2Request.options);
  }

  @Override
  public int hashCode() {
    return Objects.hash(network, atLedgerState, previewTransaction, flags, options);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class TransactionPreviewV2Request {\n");
    sb.append("    network: ").append(toIndentedString(network)).append("\n");
    sb.append("    atLedgerState: ").append(toIndentedString(atLedgerState)).append("\n");
    sb.append("    previewTransaction: ").append(toIndentedString(previewTransaction)).append("\n");
    sb.append("    flags: ").append(toIndentedString(flags)).append("\n");
    sb.append("    options: ").append(toIndentedString(options)).append("\n");
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

