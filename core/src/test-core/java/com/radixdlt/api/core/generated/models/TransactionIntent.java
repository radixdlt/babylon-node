/*
 * Radix Core API - Babylon (Bottlenose)
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.2.0
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
import com.radixdlt.api.core.generated.models.TransactionHeader;
import com.radixdlt.api.core.generated.models.TransactionMessage;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * TransactionIntent
 */
@JsonPropertyOrder({
  TransactionIntent.JSON_PROPERTY_HASH,
  TransactionIntent.JSON_PROPERTY_HASH_BECH32M,
  TransactionIntent.JSON_PROPERTY_HEADER,
  TransactionIntent.JSON_PROPERTY_INSTRUCTIONS,
  TransactionIntent.JSON_PROPERTY_BLOBS_HEX,
  TransactionIntent.JSON_PROPERTY_MESSAGE
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class TransactionIntent {
  public static final String JSON_PROPERTY_HASH = "hash";
  private String hash;

  public static final String JSON_PROPERTY_HASH_BECH32M = "hash_bech32m";
  private String hashBech32m;

  public static final String JSON_PROPERTY_HEADER = "header";
  private TransactionHeader header;

  public static final String JSON_PROPERTY_INSTRUCTIONS = "instructions";
  private String instructions;

  public static final String JSON_PROPERTY_BLOBS_HEX = "blobs_hex";
  private Map<String, String> blobsHex = null;

  public static final String JSON_PROPERTY_MESSAGE = "message";
  private TransactionMessage message;

  public TransactionIntent() { 
  }

  public TransactionIntent hash(String hash) {
    this.hash = hash;
    return this;
  }

   /**
   * The hex-encoded intent hash for a user transaction, also known as the transaction id. This hash identifies the core content \&quot;intent\&quot; of the transaction. Each intent can only be committed once. This hash gets signed by any signatories on the transaction, to create the signed intent. 
   * @return hash
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The hex-encoded intent hash for a user transaction, also known as the transaction id. This hash identifies the core content \"intent\" of the transaction. Each intent can only be committed once. This hash gets signed by any signatories on the transaction, to create the signed intent. ")
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


  public TransactionIntent hashBech32m(String hashBech32m) {
    this.hashBech32m = hashBech32m;
    return this;
  }

   /**
   * The Bech32m-encoded human readable &#x60;IntentHash&#x60;.
   * @return hashBech32m
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The Bech32m-encoded human readable `IntentHash`.")
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


  public TransactionIntent header(TransactionHeader header) {
    this.header = header;
    return this;
  }

   /**
   * Get header
   * @return header
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_HEADER)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public TransactionHeader getHeader() {
    return header;
  }


  @JsonProperty(JSON_PROPERTY_HEADER)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setHeader(TransactionHeader header) {
    this.header = header;
  }


  public TransactionIntent instructions(String instructions) {
    this.instructions = instructions;
    return this;
  }

   /**
   * The decompiled transaction manifest instructions. Only returned if enabled in &#x60;TransactionFormatOptions&#x60; on your request.
   * @return instructions
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "The decompiled transaction manifest instructions. Only returned if enabled in `TransactionFormatOptions` on your request.")
  @JsonProperty(JSON_PROPERTY_INSTRUCTIONS)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public String getInstructions() {
    return instructions;
  }


  @JsonProperty(JSON_PROPERTY_INSTRUCTIONS)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setInstructions(String instructions) {
    this.instructions = instructions;
  }


  public TransactionIntent blobsHex(Map<String, String> blobsHex) {
    this.blobsHex = blobsHex;
    return this;
  }

  public TransactionIntent putBlobsHexItem(String key, String blobsHexItem) {
    if (this.blobsHex == null) {
      this.blobsHex = new HashMap<>();
    }
    this.blobsHex.put(key, blobsHexItem);
    return this;
  }

   /**
   * A map of the hex-encoded blob hash, to hex-encoded blob content. Only returned if enabled in &#x60;TransactionFormatOptions&#x60; on your request.
   * @return blobsHex
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "A map of the hex-encoded blob hash, to hex-encoded blob content. Only returned if enabled in `TransactionFormatOptions` on your request.")
  @JsonProperty(JSON_PROPERTY_BLOBS_HEX)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public Map<String, String> getBlobsHex() {
    return blobsHex;
  }


  @JsonProperty(JSON_PROPERTY_BLOBS_HEX)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setBlobsHex(Map<String, String> blobsHex) {
    this.blobsHex = blobsHex;
  }


  public TransactionIntent message(TransactionMessage message) {
    this.message = message;
    return this;
  }

   /**
   * Get message
   * @return message
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "")
  @JsonProperty(JSON_PROPERTY_MESSAGE)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public TransactionMessage getMessage() {
    return message;
  }


  @JsonProperty(JSON_PROPERTY_MESSAGE)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setMessage(TransactionMessage message) {
    this.message = message;
  }


  /**
   * Return true if this TransactionIntent object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    TransactionIntent transactionIntent = (TransactionIntent) o;
    return Objects.equals(this.hash, transactionIntent.hash) &&
        Objects.equals(this.hashBech32m, transactionIntent.hashBech32m) &&
        Objects.equals(this.header, transactionIntent.header) &&
        Objects.equals(this.instructions, transactionIntent.instructions) &&
        Objects.equals(this.blobsHex, transactionIntent.blobsHex) &&
        Objects.equals(this.message, transactionIntent.message);
  }

  @Override
  public int hashCode() {
    return Objects.hash(hash, hashBech32m, header, instructions, blobsHex, message);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class TransactionIntent {\n");
    sb.append("    hash: ").append(toIndentedString(hash)).append("\n");
    sb.append("    hashBech32m: ").append(toIndentedString(hashBech32m)).append("\n");
    sb.append("    header: ").append(toIndentedString(header)).append("\n");
    sb.append("    instructions: ").append(toIndentedString(instructions)).append("\n");
    sb.append("    blobsHex: ").append(toIndentedString(blobsHex)).append("\n");
    sb.append("    message: ").append(toIndentedString(message)).append("\n");
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

