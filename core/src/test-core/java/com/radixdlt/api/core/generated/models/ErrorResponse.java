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
import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonSubTypes;
import com.fasterxml.jackson.annotation.JsonTypeInfo;
import com.fasterxml.jackson.annotation.JsonTypeName;
import com.fasterxml.jackson.annotation.JsonValue;
import com.radixdlt.api.core.generated.models.BasicErrorResponse;
import com.radixdlt.api.core.generated.models.ErrorResponseType;
import com.radixdlt.api.core.generated.models.LtsTransactionSubmitErrorResponse;
import com.radixdlt.api.core.generated.models.StreamProofsErrorResponse;
import com.radixdlt.api.core.generated.models.StreamTransactionsErrorResponse;
import com.radixdlt.api.core.generated.models.TransactionPreviewV2ErrorResponse;
import com.radixdlt.api.core.generated.models.TransactionSubmitErrorResponse;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


import com.radixdlt.api.core.generated.client.JSON;
/**
 * ErrorResponse
 */
@JsonPropertyOrder({
  ErrorResponse.JSON_PROPERTY_ERROR_TYPE,
  ErrorResponse.JSON_PROPERTY_CODE,
  ErrorResponse.JSON_PROPERTY_MESSAGE,
  ErrorResponse.JSON_PROPERTY_TRACE_ID
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
@JsonIgnoreProperties(
  value = "error_type", // ignore manually set error_type, it will be automatically generated by Jackson during serialization
  allowSetters = true // allows the error_type to be set during deserialization
)
@JsonTypeInfo(use = JsonTypeInfo.Id.NAME, include = JsonTypeInfo.As.PROPERTY, property = "error_type", visible = true)
@JsonSubTypes({
  @JsonSubTypes.Type(value = BasicErrorResponse.class, name = "Basic"),
  @JsonSubTypes.Type(value = BasicErrorResponse.class, name = "BasicErrorResponse"),
  @JsonSubTypes.Type(value = LtsTransactionSubmitErrorResponse.class, name = "LtsTransactionSubmit"),
  @JsonSubTypes.Type(value = LtsTransactionSubmitErrorResponse.class, name = "LtsTransactionSubmitErrorResponse"),
  @JsonSubTypes.Type(value = StreamProofsErrorResponse.class, name = "StreamProofs"),
  @JsonSubTypes.Type(value = StreamProofsErrorResponse.class, name = "StreamProofsErrorResponse"),
  @JsonSubTypes.Type(value = StreamTransactionsErrorResponse.class, name = "StreamTransactions"),
  @JsonSubTypes.Type(value = StreamTransactionsErrorResponse.class, name = "StreamTransactionsErrorResponse"),
  @JsonSubTypes.Type(value = TransactionPreviewV2ErrorResponse.class, name = "TransactionPreviewV2"),
  @JsonSubTypes.Type(value = TransactionPreviewV2ErrorResponse.class, name = "TransactionPreviewV2ErrorResponse"),
  @JsonSubTypes.Type(value = TransactionSubmitErrorResponse.class, name = "TransactionSubmit"),
  @JsonSubTypes.Type(value = TransactionSubmitErrorResponse.class, name = "TransactionSubmitErrorResponse"),
})

public class ErrorResponse {
  public static final String JSON_PROPERTY_ERROR_TYPE = "error_type";
  private ErrorResponseType errorType;

  public static final String JSON_PROPERTY_CODE = "code";
  private Integer code;

  public static final String JSON_PROPERTY_MESSAGE = "message";
  private String message;

  public static final String JSON_PROPERTY_TRACE_ID = "trace_id";
  private String traceId;

  public ErrorResponse() { 
  }

  public ErrorResponse errorType(ErrorResponseType errorType) {
    this.errorType = errorType;
    return this;
  }

   /**
   * Get errorType
   * @return errorType
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_ERROR_TYPE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public ErrorResponseType getErrorType() {
    return errorType;
  }


  @JsonProperty(JSON_PROPERTY_ERROR_TYPE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setErrorType(ErrorResponseType errorType) {
    this.errorType = errorType;
  }


  public ErrorResponse code(Integer code) {
    this.code = code;
    return this;
  }

   /**
   * A numeric code corresponding to the given HTTP error code.
   * @return code
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "A numeric code corresponding to the given HTTP error code.")
  @JsonProperty(JSON_PROPERTY_CODE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Integer getCode() {
    return code;
  }


  @JsonProperty(JSON_PROPERTY_CODE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setCode(Integer code) {
    this.code = code;
  }


  public ErrorResponse message(String message) {
    this.message = message;
    return this;
  }

   /**
   * A human-readable error message.
   * @return message
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "A human-readable error message.")
  @JsonProperty(JSON_PROPERTY_MESSAGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getMessage() {
    return message;
  }


  @JsonProperty(JSON_PROPERTY_MESSAGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setMessage(String message) {
    this.message = message;
  }


  public ErrorResponse traceId(String traceId) {
    this.traceId = traceId;
    return this;
  }

   /**
   * A GUID to be used when reporting errors, to allow correlation with the Core API&#39;s error logs, in the case where the Core API details are hidden.
   * @return traceId
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "A GUID to be used when reporting errors, to allow correlation with the Core API's error logs, in the case where the Core API details are hidden.")
  @JsonProperty(JSON_PROPERTY_TRACE_ID)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public String getTraceId() {
    return traceId;
  }


  @JsonProperty(JSON_PROPERTY_TRACE_ID)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setTraceId(String traceId) {
    this.traceId = traceId;
  }


  /**
   * Return true if this ErrorResponse object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    ErrorResponse errorResponse = (ErrorResponse) o;
    return Objects.equals(this.errorType, errorResponse.errorType) &&
        Objects.equals(this.code, errorResponse.code) &&
        Objects.equals(this.message, errorResponse.message) &&
        Objects.equals(this.traceId, errorResponse.traceId);
  }

  @Override
  public int hashCode() {
    return Objects.hash(errorType, code, message, traceId);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class ErrorResponse {\n");
    sb.append("    errorType: ").append(toIndentedString(errorType)).append("\n");
    sb.append("    code: ").append(toIndentedString(code)).append("\n");
    sb.append("    message: ").append(toIndentedString(message)).append("\n");
    sb.append("    traceId: ").append(toIndentedString(traceId)).append("\n");
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
  mappings.put("Basic", BasicErrorResponse.class);
  mappings.put("BasicErrorResponse", BasicErrorResponse.class);
  mappings.put("LtsTransactionSubmit", LtsTransactionSubmitErrorResponse.class);
  mappings.put("LtsTransactionSubmitErrorResponse", LtsTransactionSubmitErrorResponse.class);
  mappings.put("StreamProofs", StreamProofsErrorResponse.class);
  mappings.put("StreamProofsErrorResponse", StreamProofsErrorResponse.class);
  mappings.put("StreamTransactions", StreamTransactionsErrorResponse.class);
  mappings.put("StreamTransactionsErrorResponse", StreamTransactionsErrorResponse.class);
  mappings.put("TransactionPreviewV2", TransactionPreviewV2ErrorResponse.class);
  mappings.put("TransactionPreviewV2ErrorResponse", TransactionPreviewV2ErrorResponse.class);
  mappings.put("TransactionSubmit", TransactionSubmitErrorResponse.class);
  mappings.put("TransactionSubmitErrorResponse", TransactionSubmitErrorResponse.class);
  mappings.put("ErrorResponse", ErrorResponse.class);
  JSON.registerDiscriminator(ErrorResponse.class, "error_type", mappings);
}
}

