/*
 * Radix Core API
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.2.3
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
import com.radixdlt.api.core.generated.models.IntentSignatures;
import com.radixdlt.api.core.generated.models.TransactionIntentV2;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import java.util.ArrayList;
import java.util.List;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * SignedTransactionIntentV2
 */
@JsonPropertyOrder({
  SignedTransactionIntentV2.JSON_PROPERTY_HASH,
  SignedTransactionIntentV2.JSON_PROPERTY_HASH_BECH32M,
  SignedTransactionIntentV2.JSON_PROPERTY_TRANSACTION_INTENT,
  SignedTransactionIntentV2.JSON_PROPERTY_TRANSACTION_INTENT_SIGNATURES,
  SignedTransactionIntentV2.JSON_PROPERTY_NON_ROOT_SUBINTENT_SIGNATURES
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class SignedTransactionIntentV2 {
  public static final String JSON_PROPERTY_HASH = "hash";
  private String hash;

  public static final String JSON_PROPERTY_HASH_BECH32M = "hash_bech32m";
  private String hashBech32m;

  public static final String JSON_PROPERTY_TRANSACTION_INTENT = "transaction_intent";
  private TransactionIntentV2 transactionIntent;

  public static final String JSON_PROPERTY_TRANSACTION_INTENT_SIGNATURES = "transaction_intent_signatures";
  private IntentSignatures transactionIntentSignatures;

  public static final String JSON_PROPERTY_NON_ROOT_SUBINTENT_SIGNATURES = "non_root_subintent_signatures";
  private List<IntentSignatures> nonRootSubintentSignatures = new ArrayList<>();

  public SignedTransactionIntentV2() { 
  }

  public SignedTransactionIntentV2 hash(String hash) {
    this.hash = hash;
    return this;
  }

   /**
   * The hex-encoded signed intent hash for a user transaction. This hash identifies the transaction intent, plus additional signatures. This hash is signed by the notary, to create the submittable &#x60;NotarizedTransaction&#x60;. 
   * @return hash
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The hex-encoded signed intent hash for a user transaction. This hash identifies the transaction intent, plus additional signatures. This hash is signed by the notary, to create the submittable `NotarizedTransaction`. ")
  @JsonProperty(JSON_PROPERTY_HASH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getHash() {
    return hash;
  }


  @JsonProperty(JSON_PROPERTY_HASH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setHash(String hash) {
    this.hash = hash;
  }


  public SignedTransactionIntentV2 hashBech32m(String hashBech32m) {
    this.hashBech32m = hashBech32m;
    return this;
  }

   /**
   * The Bech32m-encoded human readable &#x60;SignedTransactionIntentHash&#x60;.
   * @return hashBech32m
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The Bech32m-encoded human readable `SignedTransactionIntentHash`.")
  @JsonProperty(JSON_PROPERTY_HASH_BECH32M)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getHashBech32m() {
    return hashBech32m;
  }


  @JsonProperty(JSON_PROPERTY_HASH_BECH32M)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setHashBech32m(String hashBech32m) {
    this.hashBech32m = hashBech32m;
  }


  public SignedTransactionIntentV2 transactionIntent(TransactionIntentV2 transactionIntent) {
    this.transactionIntent = transactionIntent;
    return this;
  }

   /**
   * Get transactionIntent
   * @return transactionIntent
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_TRANSACTION_INTENT)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public TransactionIntentV2 getTransactionIntent() {
    return transactionIntent;
  }


  @JsonProperty(JSON_PROPERTY_TRANSACTION_INTENT)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setTransactionIntent(TransactionIntentV2 transactionIntent) {
    this.transactionIntent = transactionIntent;
  }


  public SignedTransactionIntentV2 transactionIntentSignatures(IntentSignatures transactionIntentSignatures) {
    this.transactionIntentSignatures = transactionIntentSignatures;
    return this;
  }

   /**
   * Get transactionIntentSignatures
   * @return transactionIntentSignatures
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_TRANSACTION_INTENT_SIGNATURES)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public IntentSignatures getTransactionIntentSignatures() {
    return transactionIntentSignatures;
  }


  @JsonProperty(JSON_PROPERTY_TRANSACTION_INTENT_SIGNATURES)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setTransactionIntentSignatures(IntentSignatures transactionIntentSignatures) {
    this.transactionIntentSignatures = transactionIntentSignatures;
  }


  public SignedTransactionIntentV2 nonRootSubintentSignatures(List<IntentSignatures> nonRootSubintentSignatures) {
    this.nonRootSubintentSignatures = nonRootSubintentSignatures;
    return this;
  }

  public SignedTransactionIntentV2 addNonRootSubintentSignaturesItem(IntentSignatures nonRootSubintentSignaturesItem) {
    this.nonRootSubintentSignatures.add(nonRootSubintentSignaturesItem);
    return this;
  }

   /**
   * This gives the signatures for each subintent in &#x60;non_root_subintents&#x60; in &#x60;TransactionIntentV2&#x60;. For committed transactions, these arrays are of equal length and correspond one-to-one in order. 
   * @return nonRootSubintentSignatures
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "This gives the signatures for each subintent in `non_root_subintents` in `TransactionIntentV2`. For committed transactions, these arrays are of equal length and correspond one-to-one in order. ")
  @JsonProperty(JSON_PROPERTY_NON_ROOT_SUBINTENT_SIGNATURES)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public List<IntentSignatures> getNonRootSubintentSignatures() {
    return nonRootSubintentSignatures;
  }


  @JsonProperty(JSON_PROPERTY_NON_ROOT_SUBINTENT_SIGNATURES)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setNonRootSubintentSignatures(List<IntentSignatures> nonRootSubintentSignatures) {
    this.nonRootSubintentSignatures = nonRootSubintentSignatures;
  }


  /**
   * Return true if this SignedTransactionIntentV2 object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    SignedTransactionIntentV2 signedTransactionIntentV2 = (SignedTransactionIntentV2) o;
    return Objects.equals(this.hash, signedTransactionIntentV2.hash) &&
        Objects.equals(this.hashBech32m, signedTransactionIntentV2.hashBech32m) &&
        Objects.equals(this.transactionIntent, signedTransactionIntentV2.transactionIntent) &&
        Objects.equals(this.transactionIntentSignatures, signedTransactionIntentV2.transactionIntentSignatures) &&
        Objects.equals(this.nonRootSubintentSignatures, signedTransactionIntentV2.nonRootSubintentSignatures);
  }

  @Override
  public int hashCode() {
    return Objects.hash(hash, hashBech32m, transactionIntent, transactionIntentSignatures, nonRootSubintentSignatures);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class SignedTransactionIntentV2 {\n");
    sb.append("    hash: ").append(toIndentedString(hash)).append("\n");
    sb.append("    hashBech32m: ").append(toIndentedString(hashBech32m)).append("\n");
    sb.append("    transactionIntent: ").append(toIndentedString(transactionIntent)).append("\n");
    sb.append("    transactionIntentSignatures: ").append(toIndentedString(transactionIntentSignatures)).append("\n");
    sb.append("    nonRootSubintentSignatures: ").append(toIndentedString(nonRootSubintentSignatures)).append("\n");
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

