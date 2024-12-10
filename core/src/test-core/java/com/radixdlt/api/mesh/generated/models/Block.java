/*
 * Rosetta
 * Build Once. Integrate Your Blockchain Everywhere. 
 *
 * The version of the OpenAPI document: 1.4.13
 * 
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */


package com.radixdlt.api.mesh.generated.models;

import java.util.Objects;
import java.util.Arrays;
import java.util.Map;
import java.util.HashMap;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonTypeName;
import com.fasterxml.jackson.annotation.JsonValue;
import com.radixdlt.api.mesh.generated.models.BlockIdentifier;
import com.radixdlt.api.mesh.generated.models.Transaction;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import java.util.ArrayList;
import java.util.List;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * Blocks contain an array of Transactions that occurred at a particular BlockIdentifier.  A hard requirement for blocks returned by Rosetta implementations is that they MUST be _inalterable_: once a client has requested and received a block identified by a specific BlockIndentifier, all future calls for that same BlockIdentifier must return the same block contents. 
 */
@ApiModel(description = "Blocks contain an array of Transactions that occurred at a particular BlockIdentifier.  A hard requirement for blocks returned by Rosetta implementations is that they MUST be _inalterable_: once a client has requested and received a block identified by a specific BlockIndentifier, all future calls for that same BlockIdentifier must return the same block contents. ")
@JsonPropertyOrder({
  Block.JSON_PROPERTY_BLOCK_IDENTIFIER,
  Block.JSON_PROPERTY_PARENT_BLOCK_IDENTIFIER,
  Block.JSON_PROPERTY_TIMESTAMP,
  Block.JSON_PROPERTY_TRANSACTIONS,
  Block.JSON_PROPERTY_METADATA
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class Block {
  public static final String JSON_PROPERTY_BLOCK_IDENTIFIER = "block_identifier";
  private BlockIdentifier blockIdentifier;

  public static final String JSON_PROPERTY_PARENT_BLOCK_IDENTIFIER = "parent_block_identifier";
  private BlockIdentifier parentBlockIdentifier;

  public static final String JSON_PROPERTY_TIMESTAMP = "timestamp";
  private Long timestamp;

  public static final String JSON_PROPERTY_TRANSACTIONS = "transactions";
  private List<Transaction> transactions = new ArrayList<>();

  public static final String JSON_PROPERTY_METADATA = "metadata";
  private Object metadata;

  public Block() { 
  }

  public Block blockIdentifier(BlockIdentifier blockIdentifier) {
    this.blockIdentifier = blockIdentifier;
    return this;
  }

   /**
   * Get blockIdentifier
   * @return blockIdentifier
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_BLOCK_IDENTIFIER)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public BlockIdentifier getBlockIdentifier() {
    return blockIdentifier;
  }


  @JsonProperty(JSON_PROPERTY_BLOCK_IDENTIFIER)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setBlockIdentifier(BlockIdentifier blockIdentifier) {
    this.blockIdentifier = blockIdentifier;
  }


  public Block parentBlockIdentifier(BlockIdentifier parentBlockIdentifier) {
    this.parentBlockIdentifier = parentBlockIdentifier;
    return this;
  }

   /**
   * Get parentBlockIdentifier
   * @return parentBlockIdentifier
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_PARENT_BLOCK_IDENTIFIER)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public BlockIdentifier getParentBlockIdentifier() {
    return parentBlockIdentifier;
  }


  @JsonProperty(JSON_PROPERTY_PARENT_BLOCK_IDENTIFIER)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setParentBlockIdentifier(BlockIdentifier parentBlockIdentifier) {
    this.parentBlockIdentifier = parentBlockIdentifier;
  }


  public Block timestamp(Long timestamp) {
    this.timestamp = timestamp;
    return this;
  }

   /**
   * The timestamp of the block in milliseconds since the Unix Epoch. The timestamp is stored in milliseconds because some blockchains produce blocks more often than once a second. 
   * minimum: 0
   * @return timestamp
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(example = "1582833600000", required = true, value = "The timestamp of the block in milliseconds since the Unix Epoch. The timestamp is stored in milliseconds because some blockchains produce blocks more often than once a second. ")
  @JsonProperty(JSON_PROPERTY_TIMESTAMP)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Long getTimestamp() {
    return timestamp;
  }


  @JsonProperty(JSON_PROPERTY_TIMESTAMP)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setTimestamp(Long timestamp) {
    this.timestamp = timestamp;
  }


  public Block transactions(List<Transaction> transactions) {
    this.transactions = transactions;
    return this;
  }

  public Block addTransactionsItem(Transaction transactionsItem) {
    this.transactions.add(transactionsItem);
    return this;
  }

   /**
   * Get transactions
   * @return transactions
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_TRANSACTIONS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public List<Transaction> getTransactions() {
    return transactions;
  }


  @JsonProperty(JSON_PROPERTY_TRANSACTIONS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setTransactions(List<Transaction> transactions) {
    this.transactions = transactions;
  }


  public Block metadata(Object metadata) {
    this.metadata = metadata;
    return this;
  }

   /**
   * Get metadata
   * @return metadata
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(example = "{transactions_root=0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347, difficulty=123891724987128947}", value = "")
  @JsonProperty(JSON_PROPERTY_METADATA)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public Object getMetadata() {
    return metadata;
  }


  @JsonProperty(JSON_PROPERTY_METADATA)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setMetadata(Object metadata) {
    this.metadata = metadata;
  }


  /**
   * Return true if this Block object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    Block block = (Block) o;
    return Objects.equals(this.blockIdentifier, block.blockIdentifier) &&
        Objects.equals(this.parentBlockIdentifier, block.parentBlockIdentifier) &&
        Objects.equals(this.timestamp, block.timestamp) &&
        Objects.equals(this.transactions, block.transactions) &&
        Objects.equals(this.metadata, block.metadata);
  }

  @Override
  public int hashCode() {
    return Objects.hash(blockIdentifier, parentBlockIdentifier, timestamp, transactions, metadata);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class Block {\n");
    sb.append("    blockIdentifier: ").append(toIndentedString(blockIdentifier)).append("\n");
    sb.append("    parentBlockIdentifier: ").append(toIndentedString(parentBlockIdentifier)).append("\n");
    sb.append("    timestamp: ").append(toIndentedString(timestamp)).append("\n");
    sb.append("    transactions: ").append(toIndentedString(transactions)).append("\n");
    sb.append("    metadata: ").append(toIndentedString(metadata)).append("\n");
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
