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
import com.radixdlt.api.core.generated.models.LedgerTransactionBase;
import com.radixdlt.api.core.generated.models.LedgerTransactionType;
import com.radixdlt.api.core.generated.models.ValidatorLedgerTransactionAllOf;
import com.radixdlt.api.core.generated.models.ValidatorTransaction;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * ValidatorLedgerTransaction
 */
@JsonPropertyOrder({
  ValidatorLedgerTransaction.JSON_PROPERTY_TYPE,
  ValidatorLedgerTransaction.JSON_PROPERTY_PAYLOAD_HEX,
  ValidatorLedgerTransaction.JSON_PROPERTY_VALIDATOR_TRANSACTION
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class ValidatorLedgerTransaction {
  public static final String JSON_PROPERTY_TYPE = "type";
  private LedgerTransactionType type;

  public static final String JSON_PROPERTY_PAYLOAD_HEX = "payload_hex";
  private String payloadHex;

  public static final String JSON_PROPERTY_VALIDATOR_TRANSACTION = "validator_transaction";
  private ValidatorTransaction validatorTransaction;

  public ValidatorLedgerTransaction() { 
  }

  public ValidatorLedgerTransaction type(LedgerTransactionType type) {
    this.type = type;
    return this;
  }

   /**
   * Get type
   * @return type
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_TYPE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public LedgerTransactionType getType() {
    return type;
  }


  @JsonProperty(JSON_PROPERTY_TYPE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setType(LedgerTransactionType type) {
    this.type = type;
  }


  public ValidatorLedgerTransaction payloadHex(String payloadHex) {
    this.payloadHex = payloadHex;
    return this;
  }

   /**
   * The hex-encoded full ledger transaction payload
   * @return payloadHex
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The hex-encoded full ledger transaction payload")
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


  public ValidatorLedgerTransaction validatorTransaction(ValidatorTransaction validatorTransaction) {
    this.validatorTransaction = validatorTransaction;
    return this;
  }

   /**
   * Get validatorTransaction
   * @return validatorTransaction
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_VALIDATOR_TRANSACTION)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public ValidatorTransaction getValidatorTransaction() {
    return validatorTransaction;
  }


  @JsonProperty(JSON_PROPERTY_VALIDATOR_TRANSACTION)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setValidatorTransaction(ValidatorTransaction validatorTransaction) {
    this.validatorTransaction = validatorTransaction;
  }


  /**
   * Return true if this ValidatorLedgerTransaction object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    ValidatorLedgerTransaction validatorLedgerTransaction = (ValidatorLedgerTransaction) o;
    return Objects.equals(this.type, validatorLedgerTransaction.type) &&
        Objects.equals(this.payloadHex, validatorLedgerTransaction.payloadHex) &&
        Objects.equals(this.validatorTransaction, validatorLedgerTransaction.validatorTransaction);
  }

  @Override
  public int hashCode() {
    return Objects.hash(type, payloadHex, validatorTransaction);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class ValidatorLedgerTransaction {\n");
    sb.append("    type: ").append(toIndentedString(type)).append("\n");
    sb.append("    payloadHex: ").append(toIndentedString(payloadHex)).append("\n");
    sb.append("    validatorTransaction: ").append(toIndentedString(validatorTransaction)).append("\n");
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

