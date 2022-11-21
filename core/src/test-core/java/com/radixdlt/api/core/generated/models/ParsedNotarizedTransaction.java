/*
 * Babylon Core API
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 0.1.0
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
import com.radixdlt.api.core.generated.models.ParsedNotarizedTransactionAllOf;
import com.radixdlt.api.core.generated.models.ParsedNotarizedTransactionAllOfIdentifiers;
import com.radixdlt.api.core.generated.models.ParsedNotarizedTransactionAllOfValidationError;
import com.radixdlt.api.core.generated.models.ParsedTransactionType;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * ParsedNotarizedTransaction
 */
@JsonPropertyOrder({
  ParsedNotarizedTransaction.JSON_PROPERTY_NOTARIZED_TRANSACTION,
  ParsedNotarizedTransaction.JSON_PROPERTY_IDENTIFIERS,
  ParsedNotarizedTransaction.JSON_PROPERTY_VALIDATION_ERROR
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class ParsedNotarizedTransaction {
  public static final String JSON_PROPERTY_NOTARIZED_TRANSACTION = "notarized_transaction";
  private NotarizedTransaction notarizedTransaction;

  public static final String JSON_PROPERTY_IDENTIFIERS = "identifiers";
  private ParsedNotarizedTransactionAllOfIdentifiers identifiers;

  public static final String JSON_PROPERTY_VALIDATION_ERROR = "validation_error";
  private ParsedNotarizedTransactionAllOfValidationError validationError;

  public ParsedNotarizedTransaction() { 
  }

  public ParsedNotarizedTransaction notarizedTransaction(NotarizedTransaction notarizedTransaction) {
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


  public ParsedNotarizedTransaction identifiers(ParsedNotarizedTransactionAllOfIdentifiers identifiers) {
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

  public ParsedNotarizedTransactionAllOfIdentifiers getIdentifiers() {
    return identifiers;
  }


  @JsonProperty(JSON_PROPERTY_IDENTIFIERS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setIdentifiers(ParsedNotarizedTransactionAllOfIdentifiers identifiers) {
    this.identifiers = identifiers;
  }


  public ParsedNotarizedTransaction validationError(ParsedNotarizedTransactionAllOfValidationError validationError) {
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
   * Return true if this ParsedNotarizedTransaction object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    ParsedNotarizedTransaction parsedNotarizedTransaction = (ParsedNotarizedTransaction) o;
    return Objects.equals(this.notarizedTransaction, parsedNotarizedTransaction.notarizedTransaction) &&
        Objects.equals(this.identifiers, parsedNotarizedTransaction.identifiers) &&
        Objects.equals(this.validationError, parsedNotarizedTransaction.validationError);
  }

  @Override
  public int hashCode() {
    return Objects.hash(notarizedTransaction, identifiers, validationError);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class ParsedNotarizedTransaction {\n");
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

