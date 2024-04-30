/*
 * Radix Core API - Babylon (Anemone)
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.1.3
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
import com.radixdlt.api.core.generated.models.NotarizedTransaction;
import com.radixdlt.api.core.generated.models.ParsedNotarizedTransactionAllOfValidationError;
import com.radixdlt.api.core.generated.models.ParsedNotarizedTransactionIdentifiers;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * ParsedNotarizedTransactionAllOf
 */
@JsonPropertyOrder({
  ParsedNotarizedTransactionAllOf.JSON_PROPERTY_NOTARIZED_TRANSACTION,
  ParsedNotarizedTransactionAllOf.JSON_PROPERTY_IDENTIFIERS,
  ParsedNotarizedTransactionAllOf.JSON_PROPERTY_VALIDATION_ERROR
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class ParsedNotarizedTransactionAllOf {
  public static final String JSON_PROPERTY_NOTARIZED_TRANSACTION = "notarized_transaction";
  private NotarizedTransaction notarizedTransaction;

  public static final String JSON_PROPERTY_IDENTIFIERS = "identifiers";
  private ParsedNotarizedTransactionIdentifiers identifiers;

  public static final String JSON_PROPERTY_VALIDATION_ERROR = "validation_error";
  private ParsedNotarizedTransactionAllOfValidationError validationError;

  public ParsedNotarizedTransactionAllOf() { 
  }

  public ParsedNotarizedTransactionAllOf notarizedTransaction(NotarizedTransaction notarizedTransaction) {
    this.notarizedTransaction = notarizedTransaction;
    return this;
  }

   /**
   * Get notarizedTransaction
   * @return notarizedTransaction
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "")
  @JsonProperty(JSON_PROPERTY_NOTARIZED_TRANSACTION)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public NotarizedTransaction getNotarizedTransaction() {
    return notarizedTransaction;
  }


  @JsonProperty(JSON_PROPERTY_NOTARIZED_TRANSACTION)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setNotarizedTransaction(NotarizedTransaction notarizedTransaction) {
    this.notarizedTransaction = notarizedTransaction;
  }


  public ParsedNotarizedTransactionAllOf identifiers(ParsedNotarizedTransactionIdentifiers identifiers) {
    this.identifiers = identifiers;
    return this;
  }

   /**
   * Get identifiers
   * @return identifiers
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_IDENTIFIERS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public ParsedNotarizedTransactionIdentifiers getIdentifiers() {
    return identifiers;
  }


  @JsonProperty(JSON_PROPERTY_IDENTIFIERS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setIdentifiers(ParsedNotarizedTransactionIdentifiers identifiers) {
    this.identifiers = identifiers;
  }


  public ParsedNotarizedTransactionAllOf validationError(ParsedNotarizedTransactionAllOfValidationError validationError) {
    this.validationError = validationError;
    return this;
  }

   /**
   * Get validationError
   * @return validationError
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "")
  @JsonProperty(JSON_PROPERTY_VALIDATION_ERROR)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public ParsedNotarizedTransactionAllOfValidationError getValidationError() {
    return validationError;
  }


  @JsonProperty(JSON_PROPERTY_VALIDATION_ERROR)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setValidationError(ParsedNotarizedTransactionAllOfValidationError validationError) {
    this.validationError = validationError;
  }


  /**
   * Return true if this ParsedNotarizedTransaction_allOf object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    ParsedNotarizedTransactionAllOf parsedNotarizedTransactionAllOf = (ParsedNotarizedTransactionAllOf) o;
    return Objects.equals(this.notarizedTransaction, parsedNotarizedTransactionAllOf.notarizedTransaction) &&
        Objects.equals(this.identifiers, parsedNotarizedTransactionAllOf.identifiers) &&
        Objects.equals(this.validationError, parsedNotarizedTransactionAllOf.validationError);
  }

  @Override
  public int hashCode() {
    return Objects.hash(notarizedTransaction, identifiers, validationError);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class ParsedNotarizedTransactionAllOf {\n");
    sb.append("    notarizedTransaction: ").append(toIndentedString(notarizedTransaction)).append("\n");
    sb.append("    identifiers: ").append(toIndentedString(identifiers)).append("\n");
    sb.append("    validationError: ").append(toIndentedString(validationError)).append("\n");
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

