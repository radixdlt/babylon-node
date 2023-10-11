/*
 * Radix Core API - Babylon
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.0.4
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
 * ParsedNotarizedTransactionIdentifiers
 */
@JsonPropertyOrder({
  ParsedNotarizedTransactionIdentifiers.JSON_PROPERTY_INTENT_HASH,
  ParsedNotarizedTransactionIdentifiers.JSON_PROPERTY_INTENT_HASH_BECH32M,
  ParsedNotarizedTransactionIdentifiers.JSON_PROPERTY_SIGNED_INTENT_HASH,
  ParsedNotarizedTransactionIdentifiers.JSON_PROPERTY_SIGNED_INTENT_HASH_BECH32M,
  ParsedNotarizedTransactionIdentifiers.JSON_PROPERTY_PAYLOAD_HASH,
  ParsedNotarizedTransactionIdentifiers.JSON_PROPERTY_PAYLOAD_HASH_BECH32M,
  ParsedNotarizedTransactionIdentifiers.JSON_PROPERTY_LEDGER_HASH,
  ParsedNotarizedTransactionIdentifiers.JSON_PROPERTY_LEDGER_HASH_BECH32M
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class ParsedNotarizedTransactionIdentifiers {
  public static final String JSON_PROPERTY_INTENT_HASH = "intent_hash";
  private String intentHash;

  public static final String JSON_PROPERTY_INTENT_HASH_BECH32M = "intent_hash_bech32m";
  private String intentHashBech32m;

  public static final String JSON_PROPERTY_SIGNED_INTENT_HASH = "signed_intent_hash";
  private String signedIntentHash;

  public static final String JSON_PROPERTY_SIGNED_INTENT_HASH_BECH32M = "signed_intent_hash_bech32m";
  private String signedIntentHashBech32m;

  public static final String JSON_PROPERTY_PAYLOAD_HASH = "payload_hash";
  private String payloadHash;

  public static final String JSON_PROPERTY_PAYLOAD_HASH_BECH32M = "payload_hash_bech32m";
  private String payloadHashBech32m;

  public static final String JSON_PROPERTY_LEDGER_HASH = "ledger_hash";
  private String ledgerHash;

  public static final String JSON_PROPERTY_LEDGER_HASH_BECH32M = "ledger_hash_bech32m";
  private String ledgerHashBech32m;

  public ParsedNotarizedTransactionIdentifiers() { 
  }

  public ParsedNotarizedTransactionIdentifiers intentHash(String intentHash) {
    this.intentHash = intentHash;
    return this;
  }

   /**
   * The hex-encoded intent hash for a user transaction, also known as the transaction id. This hash identifies the core content \&quot;intent\&quot; of the transaction. Each intent can only be committed once. This hash gets signed by any signatories on the transaction, to create the signed intent. 
   * @return intentHash
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The hex-encoded intent hash for a user transaction, also known as the transaction id. This hash identifies the core content \"intent\" of the transaction. Each intent can only be committed once. This hash gets signed by any signatories on the transaction, to create the signed intent. ")
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


  public ParsedNotarizedTransactionIdentifiers intentHashBech32m(String intentHashBech32m) {
    this.intentHashBech32m = intentHashBech32m;
    return this;
  }

   /**
   * The Bech32m-encoded human readable &#x60;IntentHash&#x60;.
   * @return intentHashBech32m
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The Bech32m-encoded human readable `IntentHash`.")
  @JsonProperty(JSON_PROPERTY_INTENT_HASH_BECH32M)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getIntentHashBech32m() {
    return intentHashBech32m;
  }


  @JsonProperty(JSON_PROPERTY_INTENT_HASH_BECH32M)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setIntentHashBech32m(String intentHashBech32m) {
    this.intentHashBech32m = intentHashBech32m;
  }


  public ParsedNotarizedTransactionIdentifiers signedIntentHash(String signedIntentHash) {
    this.signedIntentHash = signedIntentHash;
    return this;
  }

   /**
   * The hex-encoded signed intent hash for a user transaction. This hash identifies the transaction intent, plus additional signatures. This hash is signed by the notary, to create the submittable NotarizedTransaction. 
   * @return signedIntentHash
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The hex-encoded signed intent hash for a user transaction. This hash identifies the transaction intent, plus additional signatures. This hash is signed by the notary, to create the submittable NotarizedTransaction. ")
  @JsonProperty(JSON_PROPERTY_SIGNED_INTENT_HASH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getSignedIntentHash() {
    return signedIntentHash;
  }


  @JsonProperty(JSON_PROPERTY_SIGNED_INTENT_HASH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setSignedIntentHash(String signedIntentHash) {
    this.signedIntentHash = signedIntentHash;
  }


  public ParsedNotarizedTransactionIdentifiers signedIntentHashBech32m(String signedIntentHashBech32m) {
    this.signedIntentHashBech32m = signedIntentHashBech32m;
    return this;
  }

   /**
   * The Bech32m-encoded human readable &#x60;SignedIntentHash&#x60;.
   * @return signedIntentHashBech32m
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The Bech32m-encoded human readable `SignedIntentHash`.")
  @JsonProperty(JSON_PROPERTY_SIGNED_INTENT_HASH_BECH32M)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getSignedIntentHashBech32m() {
    return signedIntentHashBech32m;
  }


  @JsonProperty(JSON_PROPERTY_SIGNED_INTENT_HASH_BECH32M)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setSignedIntentHashBech32m(String signedIntentHashBech32m) {
    this.signedIntentHashBech32m = signedIntentHashBech32m;
  }


  public ParsedNotarizedTransactionIdentifiers payloadHash(String payloadHash) {
    this.payloadHash = payloadHash;
    return this;
  }

   /**
   * The hex-encoded notarized transaction hash for a user transaction. This hash identifies the full submittable notarized transaction - ie the signed intent, plus the notary signature. 
   * @return payloadHash
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The hex-encoded notarized transaction hash for a user transaction. This hash identifies the full submittable notarized transaction - ie the signed intent, plus the notary signature. ")
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


  public ParsedNotarizedTransactionIdentifiers payloadHashBech32m(String payloadHashBech32m) {
    this.payloadHashBech32m = payloadHashBech32m;
    return this;
  }

   /**
   * The Bech32m-encoded human readable &#x60;NotarizedTransactionHash&#x60;.
   * @return payloadHashBech32m
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The Bech32m-encoded human readable `NotarizedTransactionHash`.")
  @JsonProperty(JSON_PROPERTY_PAYLOAD_HASH_BECH32M)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getPayloadHashBech32m() {
    return payloadHashBech32m;
  }


  @JsonProperty(JSON_PROPERTY_PAYLOAD_HASH_BECH32M)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setPayloadHashBech32m(String payloadHashBech32m) {
    this.payloadHashBech32m = payloadHashBech32m;
  }


  public ParsedNotarizedTransactionIdentifiers ledgerHash(String ledgerHash) {
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


  public ParsedNotarizedTransactionIdentifiers ledgerHashBech32m(String ledgerHashBech32m) {
    this.ledgerHashBech32m = ledgerHashBech32m;
    return this;
  }

   /**
   * The Bech32m-encoded human readable &#x60;LedgerPayloadHash&#x60;.
   * @return ledgerHashBech32m
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The Bech32m-encoded human readable `LedgerPayloadHash`.")
  @JsonProperty(JSON_PROPERTY_LEDGER_HASH_BECH32M)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getLedgerHashBech32m() {
    return ledgerHashBech32m;
  }


  @JsonProperty(JSON_PROPERTY_LEDGER_HASH_BECH32M)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setLedgerHashBech32m(String ledgerHashBech32m) {
    this.ledgerHashBech32m = ledgerHashBech32m;
  }


  /**
   * Return true if this ParsedNotarizedTransactionIdentifiers object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    ParsedNotarizedTransactionIdentifiers parsedNotarizedTransactionIdentifiers = (ParsedNotarizedTransactionIdentifiers) o;
    return Objects.equals(this.intentHash, parsedNotarizedTransactionIdentifiers.intentHash) &&
        Objects.equals(this.intentHashBech32m, parsedNotarizedTransactionIdentifiers.intentHashBech32m) &&
        Objects.equals(this.signedIntentHash, parsedNotarizedTransactionIdentifiers.signedIntentHash) &&
        Objects.equals(this.signedIntentHashBech32m, parsedNotarizedTransactionIdentifiers.signedIntentHashBech32m) &&
        Objects.equals(this.payloadHash, parsedNotarizedTransactionIdentifiers.payloadHash) &&
        Objects.equals(this.payloadHashBech32m, parsedNotarizedTransactionIdentifiers.payloadHashBech32m) &&
        Objects.equals(this.ledgerHash, parsedNotarizedTransactionIdentifiers.ledgerHash) &&
        Objects.equals(this.ledgerHashBech32m, parsedNotarizedTransactionIdentifiers.ledgerHashBech32m);
  }

  @Override
  public int hashCode() {
    return Objects.hash(intentHash, intentHashBech32m, signedIntentHash, signedIntentHashBech32m, payloadHash, payloadHashBech32m, ledgerHash, ledgerHashBech32m);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class ParsedNotarizedTransactionIdentifiers {\n");
    sb.append("    intentHash: ").append(toIndentedString(intentHash)).append("\n");
    sb.append("    intentHashBech32m: ").append(toIndentedString(intentHashBech32m)).append("\n");
    sb.append("    signedIntentHash: ").append(toIndentedString(signedIntentHash)).append("\n");
    sb.append("    signedIntentHashBech32m: ").append(toIndentedString(signedIntentHashBech32m)).append("\n");
    sb.append("    payloadHash: ").append(toIndentedString(payloadHash)).append("\n");
    sb.append("    payloadHashBech32m: ").append(toIndentedString(payloadHashBech32m)).append("\n");
    sb.append("    ledgerHash: ").append(toIndentedString(ledgerHash)).append("\n");
    sb.append("    ledgerHashBech32m: ").append(toIndentedString(ledgerHashBech32m)).append("\n");
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

