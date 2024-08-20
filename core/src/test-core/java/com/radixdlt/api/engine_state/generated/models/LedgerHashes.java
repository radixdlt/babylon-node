/*
 * Engine State API (Beta)
 * **This API is currently in Beta**  This specification may experience breaking changes as part of Babylon Node releases. Such changes will be clearly mentioned in the [babylon-node release notes](https://github.com/radixdlt/babylon-node/releases). We advise against using this API for business-critical integrations before the `version` indicated above becomes stable, which is expected in Q4 of 2024.  This API provides a complete view of the current ledger state, operating at a relatively low level (i.e. returning Entities' data and type information in a generic way, without interpreting specifics of different native or custom components).  It mirrors how the Radix Engine views the ledger state in its \"System\" layer, and thus can be useful for Scrypto developers, who need to inspect how the Engine models and stores their application's state, or how an interface / authentication scheme of another component looks like. 
 *
 * The version of the OpenAPI document: v1.2.2
 * 
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */


package com.radixdlt.api.engine_state.generated.models;

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
 * LedgerHashes
 */
@JsonPropertyOrder({
  LedgerHashes.JSON_PROPERTY_STATE_TREE_HASH,
  LedgerHashes.JSON_PROPERTY_TRANSACTION_TREE_HASH,
  LedgerHashes.JSON_PROPERTY_RECEIPT_TREE_HASH
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class LedgerHashes {
  public static final String JSON_PROPERTY_STATE_TREE_HASH = "state_tree_hash";
  private String stateTreeHash;

  public static final String JSON_PROPERTY_TRANSACTION_TREE_HASH = "transaction_tree_hash";
  private String transactionTreeHash;

  public static final String JSON_PROPERTY_RECEIPT_TREE_HASH = "receipt_tree_hash";
  private String receiptTreeHash;

  public LedgerHashes() { 
  }

  public LedgerHashes stateTreeHash(String stateTreeHash) {
    this.stateTreeHash = stateTreeHash;
    return this;
  }

   /**
   * The hex-encoded root hash of the state tree. This captures the current state of the state on the ledger. 
   * @return stateTreeHash
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The hex-encoded root hash of the state tree. This captures the current state of the state on the ledger. ")
  @JsonProperty(JSON_PROPERTY_STATE_TREE_HASH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getStateTreeHash() {
    return stateTreeHash;
  }


  @JsonProperty(JSON_PROPERTY_STATE_TREE_HASH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setStateTreeHash(String stateTreeHash) {
    this.stateTreeHash = stateTreeHash;
  }


  public LedgerHashes transactionTreeHash(String transactionTreeHash) {
    this.transactionTreeHash = transactionTreeHash;
    return this;
  }

   /**
   * The hex-encoded root hash of the transaction tree. This captures the ledger transactions committed to the ledger. 
   * @return transactionTreeHash
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The hex-encoded root hash of the transaction tree. This captures the ledger transactions committed to the ledger. ")
  @JsonProperty(JSON_PROPERTY_TRANSACTION_TREE_HASH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getTransactionTreeHash() {
    return transactionTreeHash;
  }


  @JsonProperty(JSON_PROPERTY_TRANSACTION_TREE_HASH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setTransactionTreeHash(String transactionTreeHash) {
    this.transactionTreeHash = transactionTreeHash;
  }


  public LedgerHashes receiptTreeHash(String receiptTreeHash) {
    this.receiptTreeHash = receiptTreeHash;
    return this;
  }

   /**
   * The hex-encoded root hash of the receipt tree. This captures the consensus-agreed output of each transaction on the ledger. 
   * @return receiptTreeHash
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The hex-encoded root hash of the receipt tree. This captures the consensus-agreed output of each transaction on the ledger. ")
  @JsonProperty(JSON_PROPERTY_RECEIPT_TREE_HASH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getReceiptTreeHash() {
    return receiptTreeHash;
  }


  @JsonProperty(JSON_PROPERTY_RECEIPT_TREE_HASH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setReceiptTreeHash(String receiptTreeHash) {
    this.receiptTreeHash = receiptTreeHash;
  }


  /**
   * Return true if this LedgerHashes object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    LedgerHashes ledgerHashes = (LedgerHashes) o;
    return Objects.equals(this.stateTreeHash, ledgerHashes.stateTreeHash) &&
        Objects.equals(this.transactionTreeHash, ledgerHashes.transactionTreeHash) &&
        Objects.equals(this.receiptTreeHash, ledgerHashes.receiptTreeHash);
  }

  @Override
  public int hashCode() {
    return Objects.hash(stateTreeHash, transactionTreeHash, receiptTreeHash);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class LedgerHashes {\n");
    sb.append("    stateTreeHash: ").append(toIndentedString(stateTreeHash)).append("\n");
    sb.append("    transactionTreeHash: ").append(toIndentedString(transactionTreeHash)).append("\n");
    sb.append("    receiptTreeHash: ").append(toIndentedString(receiptTreeHash)).append("\n");
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

