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
import com.radixdlt.api.core.generated.models.TransactionFormatOptions;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * TransactionParseRequest
 */
@JsonPropertyOrder({
  TransactionParseRequest.JSON_PROPERTY_NETWORK,
  TransactionParseRequest.JSON_PROPERTY_PAYLOAD_HEX,
  TransactionParseRequest.JSON_PROPERTY_PARSE_MODE,
  TransactionParseRequest.JSON_PROPERTY_VALIDATION_MODE,
  TransactionParseRequest.JSON_PROPERTY_RESPONSE_MODE,
  TransactionParseRequest.JSON_PROPERTY_TRANSACTION_FORMAT_OPTIONS
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class TransactionParseRequest {
  public static final String JSON_PROPERTY_NETWORK = "network";
  private String network;

  public static final String JSON_PROPERTY_PAYLOAD_HEX = "payload_hex";
  private String payloadHex;

  /**
   * The type of transaction payload that should be assumed. If omitted, \&quot;Any\&quot; is used - where the payload is attempted to be parsed as each of the following in turn: Notarized, Signed, Unsigned, Ledger. 
   */
  public enum ParseModeEnum {
    ANY("Any"),
    
    NOTARIZED("Notarized"),
    
    SIGNED("Signed"),
    
    UNSIGNED("Unsigned"),
    
    LEDGER("Ledger");

    private String value;

    ParseModeEnum(String value) {
      this.value = value;
    }

    @JsonValue
    public String getValue() {
      return value;
    }

    @Override
    public String toString() {
      return String.valueOf(value);
    }

    @JsonCreator
    public static ParseModeEnum fromValue(String value) {
      for (ParseModeEnum b : ParseModeEnum.values()) {
        if (b.value.equals(value)) {
          return b;
        }
      }
      throw new IllegalArgumentException("Unexpected value '" + value + "'");
    }
  }

  public static final String JSON_PROPERTY_PARSE_MODE = "parse_mode";
  private ParseModeEnum parseMode;

  /**
   * The type of validation that should be performed, if the payload correctly decompiles as a Notarized Transaction. This is only relevant for Notarized payloads. If omitted, \&quot;Static\&quot; is used. 
   */
  public enum ValidationModeEnum {
    NONE("None"),
    
    STATIC("Static"),
    
    FULL("Full");

    private String value;

    ValidationModeEnum(String value) {
      this.value = value;
    }

    @JsonValue
    public String getValue() {
      return value;
    }

    @Override
    public String toString() {
      return String.valueOf(value);
    }

    @JsonCreator
    public static ValidationModeEnum fromValue(String value) {
      for (ValidationModeEnum b : ValidationModeEnum.values()) {
        if (b.value.equals(value)) {
          return b;
        }
      }
      throw new IllegalArgumentException("Unexpected value '" + value + "'");
    }
  }

  public static final String JSON_PROPERTY_VALIDATION_MODE = "validation_mode";
  private ValidationModeEnum validationMode;

  /**
   * The amount of information to return in the response. \&quot;Basic\&quot; includes the type, validity information, and any relevant identifiers. \&quot;Full\&quot; also includes the fully parsed information. If omitted, \&quot;Full\&quot; is used. 
   */
  public enum ResponseModeEnum {
    BASIC("Basic"),
    
    FULL("Full");

    private String value;

    ResponseModeEnum(String value) {
      this.value = value;
    }

    @JsonValue
    public String getValue() {
      return value;
    }

    @Override
    public String toString() {
      return String.valueOf(value);
    }

    @JsonCreator
    public static ResponseModeEnum fromValue(String value) {
      for (ResponseModeEnum b : ResponseModeEnum.values()) {
        if (b.value.equals(value)) {
          return b;
        }
      }
      throw new IllegalArgumentException("Unexpected value '" + value + "'");
    }
  }

  public static final String JSON_PROPERTY_RESPONSE_MODE = "response_mode";
  private ResponseModeEnum responseMode;

  public static final String JSON_PROPERTY_TRANSACTION_FORMAT_OPTIONS = "transaction_format_options";
  private TransactionFormatOptions transactionFormatOptions;

  public TransactionParseRequest() { 
  }

  public TransactionParseRequest network(String network) {
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


  public TransactionParseRequest payloadHex(String payloadHex) {
    this.payloadHex = payloadHex;
    return this;
  }

   /**
   * A hex-encoded payload of a full transaction or a partial transaction - either a notarized transaction, a signed transaction intent an unsigned transaction intent, or a ledger payload. 
   * @return payloadHex
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "A hex-encoded payload of a full transaction or a partial transaction - either a notarized transaction, a signed transaction intent an unsigned transaction intent, or a ledger payload. ")
  @JsonProperty(JSON_PROPERTY_PAYLOAD_HEX)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getPayloadHex() {
    return payloadHex;
  }


  @JsonProperty(JSON_PROPERTY_PAYLOAD_HEX)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setPayloadHex(String payloadHex) {
    this.payloadHex = payloadHex;
  }


  public TransactionParseRequest parseMode(ParseModeEnum parseMode) {
    this.parseMode = parseMode;
    return this;
  }

   /**
   * The type of transaction payload that should be assumed. If omitted, \&quot;Any\&quot; is used - where the payload is attempted to be parsed as each of the following in turn: Notarized, Signed, Unsigned, Ledger. 
   * @return parseMode
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "The type of transaction payload that should be assumed. If omitted, \"Any\" is used - where the payload is attempted to be parsed as each of the following in turn: Notarized, Signed, Unsigned, Ledger. ")
  @JsonProperty(JSON_PROPERTY_PARSE_MODE)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public ParseModeEnum getParseMode() {
    return parseMode;
  }


  @JsonProperty(JSON_PROPERTY_PARSE_MODE)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setParseMode(ParseModeEnum parseMode) {
    this.parseMode = parseMode;
  }


  public TransactionParseRequest validationMode(ValidationModeEnum validationMode) {
    this.validationMode = validationMode;
    return this;
  }

   /**
   * The type of validation that should be performed, if the payload correctly decompiles as a Notarized Transaction. This is only relevant for Notarized payloads. If omitted, \&quot;Static\&quot; is used. 
   * @return validationMode
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "The type of validation that should be performed, if the payload correctly decompiles as a Notarized Transaction. This is only relevant for Notarized payloads. If omitted, \"Static\" is used. ")
  @JsonProperty(JSON_PROPERTY_VALIDATION_MODE)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public ValidationModeEnum getValidationMode() {
    return validationMode;
  }


  @JsonProperty(JSON_PROPERTY_VALIDATION_MODE)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setValidationMode(ValidationModeEnum validationMode) {
    this.validationMode = validationMode;
  }


  public TransactionParseRequest responseMode(ResponseModeEnum responseMode) {
    this.responseMode = responseMode;
    return this;
  }

   /**
   * The amount of information to return in the response. \&quot;Basic\&quot; includes the type, validity information, and any relevant identifiers. \&quot;Full\&quot; also includes the fully parsed information. If omitted, \&quot;Full\&quot; is used. 
   * @return responseMode
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "The amount of information to return in the response. \"Basic\" includes the type, validity information, and any relevant identifiers. \"Full\" also includes the fully parsed information. If omitted, \"Full\" is used. ")
  @JsonProperty(JSON_PROPERTY_RESPONSE_MODE)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public ResponseModeEnum getResponseMode() {
    return responseMode;
  }


  @JsonProperty(JSON_PROPERTY_RESPONSE_MODE)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setResponseMode(ResponseModeEnum responseMode) {
    this.responseMode = responseMode;
  }


  public TransactionParseRequest transactionFormatOptions(TransactionFormatOptions transactionFormatOptions) {
    this.transactionFormatOptions = transactionFormatOptions;
    return this;
  }

   /**
   * Get transactionFormatOptions
   * @return transactionFormatOptions
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "")
  @JsonProperty(JSON_PROPERTY_TRANSACTION_FORMAT_OPTIONS)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public TransactionFormatOptions getTransactionFormatOptions() {
    return transactionFormatOptions;
  }


  @JsonProperty(JSON_PROPERTY_TRANSACTION_FORMAT_OPTIONS)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setTransactionFormatOptions(TransactionFormatOptions transactionFormatOptions) {
    this.transactionFormatOptions = transactionFormatOptions;
  }


  /**
   * Return true if this TransactionParseRequest object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    TransactionParseRequest transactionParseRequest = (TransactionParseRequest) o;
    return Objects.equals(this.network, transactionParseRequest.network) &&
        Objects.equals(this.payloadHex, transactionParseRequest.payloadHex) &&
        Objects.equals(this.parseMode, transactionParseRequest.parseMode) &&
        Objects.equals(this.validationMode, transactionParseRequest.validationMode) &&
        Objects.equals(this.responseMode, transactionParseRequest.responseMode) &&
        Objects.equals(this.transactionFormatOptions, transactionParseRequest.transactionFormatOptions);
  }

  @Override
  public int hashCode() {
    return Objects.hash(network, payloadHex, parseMode, validationMode, responseMode, transactionFormatOptions);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class TransactionParseRequest {\n");
    sb.append("    network: ").append(toIndentedString(network)).append("\n");
    sb.append("    payloadHex: ").append(toIndentedString(payloadHex)).append("\n");
    sb.append("    parseMode: ").append(toIndentedString(parseMode)).append("\n");
    sb.append("    validationMode: ").append(toIndentedString(validationMode)).append("\n");
    sb.append("    responseMode: ").append(toIndentedString(responseMode)).append("\n");
    sb.append("    transactionFormatOptions: ").append(toIndentedString(transactionFormatOptions)).append("\n");
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

