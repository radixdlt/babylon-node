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
import com.radixdlt.api.core.generated.models.InstructionResourceChanges;
import com.radixdlt.api.core.generated.models.LedgerStateSummary;
import com.radixdlt.api.core.generated.models.TransactionPreviewResponseLogsInner;
import com.radixdlt.api.core.generated.models.TransactionReceipt;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import java.util.ArrayList;
import java.util.List;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * TransactionPreviewResponse
 */
@JsonPropertyOrder({
  TransactionPreviewResponse.JSON_PROPERTY_AT_LEDGER_STATE,
  TransactionPreviewResponse.JSON_PROPERTY_ENCODED_RECEIPT,
  TransactionPreviewResponse.JSON_PROPERTY_RECEIPT,
  TransactionPreviewResponse.JSON_PROPERTY_RADIX_ENGINE_TOOLKIT_RECEIPT,
  TransactionPreviewResponse.JSON_PROPERTY_INSTRUCTION_RESOURCE_CHANGES,
  TransactionPreviewResponse.JSON_PROPERTY_LOGS
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class TransactionPreviewResponse {
  public static final String JSON_PROPERTY_AT_LEDGER_STATE = "at_ledger_state";
  private LedgerStateSummary atLedgerState;

  public static final String JSON_PROPERTY_ENCODED_RECEIPT = "encoded_receipt";
  private String encodedReceipt;

  public static final String JSON_PROPERTY_RECEIPT = "receipt";
  private TransactionReceipt receipt;

  public static final String JSON_PROPERTY_RADIX_ENGINE_TOOLKIT_RECEIPT = "radix_engine_toolkit_receipt";
  private Object radixEngineToolkitReceipt;

  public static final String JSON_PROPERTY_INSTRUCTION_RESOURCE_CHANGES = "instruction_resource_changes";
  private List<InstructionResourceChanges> instructionResourceChanges = new ArrayList<>();

  public static final String JSON_PROPERTY_LOGS = "logs";
  private List<TransactionPreviewResponseLogsInner> logs = new ArrayList<>();

  public TransactionPreviewResponse() { 
  }

  public TransactionPreviewResponse atLedgerState(LedgerStateSummary atLedgerState) {
    this.atLedgerState = atLedgerState;
    return this;
  }

   /**
   * Get atLedgerState
   * @return atLedgerState
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_AT_LEDGER_STATE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public LedgerStateSummary getAtLedgerState() {
    return atLedgerState;
  }


  @JsonProperty(JSON_PROPERTY_AT_LEDGER_STATE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setAtLedgerState(LedgerStateSummary atLedgerState) {
    this.atLedgerState = atLedgerState;
  }


  public TransactionPreviewResponse encodedReceipt(String encodedReceipt) {
    this.encodedReceipt = encodedReceipt;
    return this;
  }

   /**
   * The hex-sbor-encoded receipt.  This field is deprecated and will be removed from the API with the release of the next  protocol update, cuttlefish. This field was provided primarily for use with the Radix  Engine Toolkit and its execution summary functionality. If you still wish to use this  functionality update your Radix Engine Toolkit and use the receipt provided in the  &#x60;radix_engine_toolkit_receipt&#x60; field of this response. 
   * @return encodedReceipt
   * @deprecated
  **/
  @Deprecated
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The hex-sbor-encoded receipt.  This field is deprecated and will be removed from the API with the release of the next  protocol update, cuttlefish. This field was provided primarily for use with the Radix  Engine Toolkit and its execution summary functionality. If you still wish to use this  functionality update your Radix Engine Toolkit and use the receipt provided in the  `radix_engine_toolkit_receipt` field of this response. ")
  @JsonProperty(JSON_PROPERTY_ENCODED_RECEIPT)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getEncodedReceipt() {
    return encodedReceipt;
  }


  @JsonProperty(JSON_PROPERTY_ENCODED_RECEIPT)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setEncodedReceipt(String encodedReceipt) {
    this.encodedReceipt = encodedReceipt;
  }


  public TransactionPreviewResponse receipt(TransactionReceipt receipt) {
    this.receipt = receipt;
    return this;
  }

   /**
   * Get receipt
   * @return receipt
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_RECEIPT)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public TransactionReceipt getReceipt() {
    return receipt;
  }


  @JsonProperty(JSON_PROPERTY_RECEIPT)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setReceipt(TransactionReceipt receipt) {
    this.receipt = receipt;
  }


  public TransactionPreviewResponse radixEngineToolkitReceipt(Object radixEngineToolkitReceipt) {
    this.radixEngineToolkitReceipt = radixEngineToolkitReceipt;
    return this;
  }

   /**
   * An optional field which is only provided if the &#x60;radix_engine_toolkit_receipt&#x60; flag is set to true when requesting a transaction preview from the API.  This receipt is primarily intended for use with the toolkit and may contain information  that is already available in the receipt provided in the &#x60;receipt&#x60; field of this  response.  A typical client of this API is not expected to use this receipt. The primary clients  this receipt is intended for is the Radix wallet or any client that needs to perform  execution summaries on their transactions. 
   * @return radixEngineToolkitReceipt
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "An optional field which is only provided if the `radix_engine_toolkit_receipt` flag is set to true when requesting a transaction preview from the API.  This receipt is primarily intended for use with the toolkit and may contain information  that is already available in the receipt provided in the `receipt` field of this  response.  A typical client of this API is not expected to use this receipt. The primary clients  this receipt is intended for is the Radix wallet or any client that needs to perform  execution summaries on their transactions. ")
  @JsonProperty(JSON_PROPERTY_RADIX_ENGINE_TOOLKIT_RECEIPT)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public Object getRadixEngineToolkitReceipt() {
    return radixEngineToolkitReceipt;
  }


  @JsonProperty(JSON_PROPERTY_RADIX_ENGINE_TOOLKIT_RECEIPT)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setRadixEngineToolkitReceipt(Object radixEngineToolkitReceipt) {
    this.radixEngineToolkitReceipt = radixEngineToolkitReceipt;
  }


  public TransactionPreviewResponse instructionResourceChanges(List<InstructionResourceChanges> instructionResourceChanges) {
    this.instructionResourceChanges = instructionResourceChanges;
    return this;
  }

  public TransactionPreviewResponse addInstructionResourceChangesItem(InstructionResourceChanges instructionResourceChangesItem) {
    this.instructionResourceChanges.add(instructionResourceChangesItem);
    return this;
  }

   /**
   * Get instructionResourceChanges
   * @return instructionResourceChanges
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_INSTRUCTION_RESOURCE_CHANGES)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public List<InstructionResourceChanges> getInstructionResourceChanges() {
    return instructionResourceChanges;
  }


  @JsonProperty(JSON_PROPERTY_INSTRUCTION_RESOURCE_CHANGES)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setInstructionResourceChanges(List<InstructionResourceChanges> instructionResourceChanges) {
    this.instructionResourceChanges = instructionResourceChanges;
  }


  public TransactionPreviewResponse logs(List<TransactionPreviewResponseLogsInner> logs) {
    this.logs = logs;
    return this;
  }

  public TransactionPreviewResponse addLogsItem(TransactionPreviewResponseLogsInner logsItem) {
    this.logs.add(logsItem);
    return this;
  }

   /**
   * Get logs
   * @return logs
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_LOGS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public List<TransactionPreviewResponseLogsInner> getLogs() {
    return logs;
  }


  @JsonProperty(JSON_PROPERTY_LOGS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setLogs(List<TransactionPreviewResponseLogsInner> logs) {
    this.logs = logs;
  }


  /**
   * Return true if this TransactionPreviewResponse object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    TransactionPreviewResponse transactionPreviewResponse = (TransactionPreviewResponse) o;
    return Objects.equals(this.atLedgerState, transactionPreviewResponse.atLedgerState) &&
        Objects.equals(this.encodedReceipt, transactionPreviewResponse.encodedReceipt) &&
        Objects.equals(this.receipt, transactionPreviewResponse.receipt) &&
        Objects.equals(this.radixEngineToolkitReceipt, transactionPreviewResponse.radixEngineToolkitReceipt) &&
        Objects.equals(this.instructionResourceChanges, transactionPreviewResponse.instructionResourceChanges) &&
        Objects.equals(this.logs, transactionPreviewResponse.logs);
  }

  @Override
  public int hashCode() {
    return Objects.hash(atLedgerState, encodedReceipt, receipt, radixEngineToolkitReceipt, instructionResourceChanges, logs);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class TransactionPreviewResponse {\n");
    sb.append("    atLedgerState: ").append(toIndentedString(atLedgerState)).append("\n");
    sb.append("    encodedReceipt: ").append(toIndentedString(encodedReceipt)).append("\n");
    sb.append("    receipt: ").append(toIndentedString(receipt)).append("\n");
    sb.append("    radixEngineToolkitReceipt: ").append(toIndentedString(radixEngineToolkitReceipt)).append("\n");
    sb.append("    instructionResourceChanges: ").append(toIndentedString(instructionResourceChanges)).append("\n");
    sb.append("    logs: ").append(toIndentedString(logs)).append("\n");
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

