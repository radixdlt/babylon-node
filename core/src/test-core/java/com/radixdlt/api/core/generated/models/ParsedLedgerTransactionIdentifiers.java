/*
 * Babylon Core API - RCnet v3
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  This version of the Core API belongs to the second release candidate of the Radix Babylon network (\"RCnet v3\").  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` are guaranteed to be forward compatible to Babylon mainnet launch (and beyond). We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code. 
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
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * ParsedLedgerTransactionIdentifiers
 */
@JsonPropertyOrder({
  ParsedLedgerTransactionIdentifiers.JSON_PROPERTY_INTENT_HASH,
  ParsedLedgerTransactionIdentifiers.JSON_PROPERTY_SIGNED_INTENT_HASH,
  ParsedLedgerTransactionIdentifiers.JSON_PROPERTY_PAYLOAD_HASH,
  ParsedLedgerTransactionIdentifiers.JSON_PROPERTY_LEDGER_HASH
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class ParsedLedgerTransactionIdentifiers {
  public static final String JSON_PROPERTY_INTENT_HASH = "intent_hash";
  private String intentHash;

  public static final String JSON_PROPERTY_SIGNED_INTENT_HASH = "signed_intent_hash";
  private String signedIntentHash;

  public static final String JSON_PROPERTY_PAYLOAD_HASH = "payload_hash";
  private String payloadHash;

  public static final String JSON_PROPERTY_LEDGER_HASH = "ledger_hash";
  private String ledgerHash;

  public ParsedLedgerTransactionIdentifiers() { 
  }

  public ParsedLedgerTransactionIdentifiers intentHash(String intentHash) {
    this.intentHash = intentHash;
    return this;
  }

   /**
   * The hex-encoded intent hash for a user transaction, also known as the transaction id. This hash identifies the core content \&quot;intent\&quot; of the transaction. Each intent can only be committed once. This hash gets signed by any signatories on the transaction, to create the signed intent. 
   * @return intentHash
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "The hex-encoded intent hash for a user transaction, also known as the transaction id. This hash identifies the core content \"intent\" of the transaction. Each intent can only be committed once. This hash gets signed by any signatories on the transaction, to create the signed intent. ")
  @JsonProperty(JSON_PROPERTY_INTENT_HASH)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public String getIntentHash() {
    return intentHash;
  }


  @JsonProperty(JSON_PROPERTY_INTENT_HASH)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setIntentHash(String intentHash) {
    this.intentHash = intentHash;
  }


  public ParsedLedgerTransactionIdentifiers signedIntentHash(String signedIntentHash) {
    this.signedIntentHash = signedIntentHash;
    return this;
  }

   /**
   * The hex-encoded signed intent hash for a user transaction. This hash identifies the transaction intent, plus additional signatures. This hash is signed by the notary, to create the submittable NotarizedTransaction. 
   * @return signedIntentHash
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "The hex-encoded signed intent hash for a user transaction. This hash identifies the transaction intent, plus additional signatures. This hash is signed by the notary, to create the submittable NotarizedTransaction. ")
  @JsonProperty(JSON_PROPERTY_SIGNED_INTENT_HASH)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public String getSignedIntentHash() {
    return signedIntentHash;
  }


  @JsonProperty(JSON_PROPERTY_SIGNED_INTENT_HASH)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setSignedIntentHash(String signedIntentHash) {
    this.signedIntentHash = signedIntentHash;
  }


  public ParsedLedgerTransactionIdentifiers payloadHash(String payloadHash) {
    this.payloadHash = payloadHash;
    return this;
  }

   /**
   * The hex-encoded notarized transaction hash for a user transaction. This hash identifies the full submittable notarized transaction - ie the signed intent, plus the notary signature. 
   * @return payloadHash
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "The hex-encoded notarized transaction hash for a user transaction. This hash identifies the full submittable notarized transaction - ie the signed intent, plus the notary signature. ")
  @JsonProperty(JSON_PROPERTY_PAYLOAD_HASH)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public String getPayloadHash() {
    return payloadHash;
  }


  @JsonProperty(JSON_PROPERTY_PAYLOAD_HASH)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setPayloadHash(String payloadHash) {
    this.payloadHash = payloadHash;
  }


  public ParsedLedgerTransactionIdentifiers ledgerHash(String ledgerHash) {
    this.ledgerHash = ledgerHash;
    return this;
  }

   /**
   * The hex-encoded ledger payload transaction hash. This is a wrapper for both user transactions, and system transactions such as genesis and round changes. 
   * @return ledgerHash
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The hex-encoded ledger payload transaction hash. This is a wrapper for both user transactions, and system transactions such as genesis and round changes. ")
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
   * Return true if this ParsedLedgerTransactionIdentifiers object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    ParsedLedgerTransactionIdentifiers parsedLedgerTransactionIdentifiers = (ParsedLedgerTransactionIdentifiers) o;
    return Objects.equals(this.intentHash, parsedLedgerTransactionIdentifiers.intentHash) &&
        Objects.equals(this.signedIntentHash, parsedLedgerTransactionIdentifiers.signedIntentHash) &&
        Objects.equals(this.payloadHash, parsedLedgerTransactionIdentifiers.payloadHash) &&
        Objects.equals(this.ledgerHash, parsedLedgerTransactionIdentifiers.ledgerHash);
  }

  @Override
  public int hashCode() {
    return Objects.hash(intentHash, signedIntentHash, payloadHash, ledgerHash);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class ParsedLedgerTransactionIdentifiers {\n");
    sb.append("    intentHash: ").append(toIndentedString(intentHash)).append("\n");
    sb.append("    signedIntentHash: ").append(toIndentedString(signedIntentHash)).append("\n");
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

