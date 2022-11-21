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
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * ParsedNotarizedTransactionAllOfIdentifiers
 */
@JsonPropertyOrder({
  ParsedNotarizedTransactionAllOfIdentifiers.JSON_PROPERTY_INTENT_HASH,
  ParsedNotarizedTransactionAllOfIdentifiers.JSON_PROPERTY_SIGNATURES_HASH,
  ParsedNotarizedTransactionAllOfIdentifiers.JSON_PROPERTY_PAYLOAD_HASH,
  ParsedNotarizedTransactionAllOfIdentifiers.JSON_PROPERTY_LEDGER_HASH
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class ParsedNotarizedTransactionAllOfIdentifiers {
  public static final String JSON_PROPERTY_INTENT_HASH = "intent_hash";
  private String intentHash;

  public static final String JSON_PROPERTY_SIGNATURES_HASH = "signatures_hash";
  private String signaturesHash;

  public static final String JSON_PROPERTY_PAYLOAD_HASH = "payload_hash";
  private String payloadHash;

  public static final String JSON_PROPERTY_LEDGER_HASH = "ledger_hash";
  private String ledgerHash;

  public ParsedNotarizedTransactionAllOfIdentifiers() { 
  }

  public ParsedNotarizedTransactionAllOfIdentifiers intentHash(String intentHash) {
    this.intentHash = intentHash;
    return this;
  }

   /**
   * The hex-encoded transaction intent hash. This is known as the Intent Hash, Transaction ID or Transaction Identifier for user transactions. This hash is &#x60;SHA256(SHA256(compiled_intent))&#x60;
   * @return intentHash
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The hex-encoded transaction intent hash. This is known as the Intent Hash, Transaction ID or Transaction Identifier for user transactions. This hash is `SHA256(SHA256(compiled_intent))`")
  @JsonProperty(JSON_PROPERTY_INTENT_HASH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getIntentHash() {
    return intentHash;
  }


  @JsonProperty(JSON_PROPERTY_INTENT_HASH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setIntentHash(String intentHash) {
    this.intentHash = intentHash;
  }


  public ParsedNotarizedTransactionAllOfIdentifiers signaturesHash(String signaturesHash) {
    this.signaturesHash = signaturesHash;
    return this;
  }

   /**
   * The hex-encoded signed transaction hash. This is known as the Signed Transaction Hash or Signatures Hash. This is the hash which is signed as part of notarization. This hash is &#x60;SHA256(SHA256(compiled_signed_transaction))&#x60;
   * @return signaturesHash
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The hex-encoded signed transaction hash. This is known as the Signed Transaction Hash or Signatures Hash. This is the hash which is signed as part of notarization. This hash is `SHA256(SHA256(compiled_signed_transaction))`")
  @JsonProperty(JSON_PROPERTY_SIGNATURES_HASH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getSignaturesHash() {
    return signaturesHash;
  }


  @JsonProperty(JSON_PROPERTY_SIGNATURES_HASH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setSignaturesHash(String signaturesHash) {
    this.signaturesHash = signaturesHash;
  }


  public ParsedNotarizedTransactionAllOfIdentifiers payloadHash(String payloadHash) {
    this.payloadHash = payloadHash;
    return this;
  }

   /**
   * The hex-encoded notarized transaction hash. This is known as the Notarized Transaction Hash, Payload Hash or User Payload Hash. This hash is &#x60;SHA256(SHA256(compiled_notarized_transaction))&#x60;
   * @return payloadHash
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The hex-encoded notarized transaction hash. This is known as the Notarized Transaction Hash, Payload Hash or User Payload Hash. This hash is `SHA256(SHA256(compiled_notarized_transaction))`")
  @JsonProperty(JSON_PROPERTY_PAYLOAD_HASH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getPayloadHash() {
    return payloadHash;
  }


  @JsonProperty(JSON_PROPERTY_PAYLOAD_HASH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setPayloadHash(String payloadHash) {
    this.payloadHash = payloadHash;
  }


  public ParsedNotarizedTransactionAllOfIdentifiers ledgerHash(String ledgerHash) {
    this.ledgerHash = ledgerHash;
    return this;
  }

   /**
   * The hex-encoded ledger-wrapped transaction hash. This is known as the Ledger Hash. This hash is &#x60;SHA256(SHA256(ledger_transaction_bytes))&#x60;
   * @return ledgerHash
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The hex-encoded ledger-wrapped transaction hash. This is known as the Ledger Hash. This hash is `SHA256(SHA256(ledger_transaction_bytes))`")
  @JsonProperty(JSON_PROPERTY_LEDGER_HASH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getLedgerHash() {
    return ledgerHash;
  }


  @JsonProperty(JSON_PROPERTY_LEDGER_HASH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setLedgerHash(String ledgerHash) {
    this.ledgerHash = ledgerHash;
  }


  /**
   * Return true if this ParsedNotarizedTransaction_allOf_identifiers object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    ParsedNotarizedTransactionAllOfIdentifiers parsedNotarizedTransactionAllOfIdentifiers = (ParsedNotarizedTransactionAllOfIdentifiers) o;
    return Objects.equals(this.intentHash, parsedNotarizedTransactionAllOfIdentifiers.intentHash) &&
        Objects.equals(this.signaturesHash, parsedNotarizedTransactionAllOfIdentifiers.signaturesHash) &&
        Objects.equals(this.payloadHash, parsedNotarizedTransactionAllOfIdentifiers.payloadHash) &&
        Objects.equals(this.ledgerHash, parsedNotarizedTransactionAllOfIdentifiers.ledgerHash);
  }

  @Override
  public int hashCode() {
    return Objects.hash(intentHash, signaturesHash, payloadHash, ledgerHash);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class ParsedNotarizedTransactionAllOfIdentifiers {\n");
    sb.append("    intentHash: ").append(toIndentedString(intentHash)).append("\n");
    sb.append("    signaturesHash: ").append(toIndentedString(signaturesHash)).append("\n");
    sb.append("    payloadHash: ").append(toIndentedString(payloadHash)).append("\n");
    sb.append("    ledgerHash: ").append(toIndentedString(ledgerHash)).append("\n");
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

