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
import com.radixdlt.api.mesh.generated.models.Operation;
import com.radixdlt.api.mesh.generated.models.RelatedTransaction;
import com.radixdlt.api.mesh.generated.models.TransactionIdentifier;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import java.util.ArrayList;
import java.util.List;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * Transactions contain an array of Operations that are attributable to the same TransactionIdentifier. 
 */
@ApiModel(description = "Transactions contain an array of Operations that are attributable to the same TransactionIdentifier. ")
@JsonPropertyOrder({
  Transaction.JSON_PROPERTY_TRANSACTION_IDENTIFIER,
  Transaction.JSON_PROPERTY_OPERATIONS,
  Transaction.JSON_PROPERTY_RELATED_TRANSACTIONS,
  Transaction.JSON_PROPERTY_METADATA
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class Transaction {
  public static final String JSON_PROPERTY_TRANSACTION_IDENTIFIER = "transaction_identifier";
  private TransactionIdentifier transactionIdentifier;

  public static final String JSON_PROPERTY_OPERATIONS = "operations";
  private List<Operation> operations = new ArrayList<>();

  public static final String JSON_PROPERTY_RELATED_TRANSACTIONS = "related_transactions";
  private List<RelatedTransaction> relatedTransactions = null;

  public static final String JSON_PROPERTY_METADATA = "metadata";
  private Object metadata;

  public Transaction() { 
  }

  public Transaction transactionIdentifier(TransactionIdentifier transactionIdentifier) {
    this.transactionIdentifier = transactionIdentifier;
    return this;
  }

   /**
   * Get transactionIdentifier
   * @return transactionIdentifier
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_TRANSACTION_IDENTIFIER)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public TransactionIdentifier getTransactionIdentifier() {
    return transactionIdentifier;
  }


  @JsonProperty(JSON_PROPERTY_TRANSACTION_IDENTIFIER)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setTransactionIdentifier(TransactionIdentifier transactionIdentifier) {
    this.transactionIdentifier = transactionIdentifier;
  }


  public Transaction operations(List<Operation> operations) {
    this.operations = operations;
    return this;
  }

  public Transaction addOperationsItem(Operation operationsItem) {
    this.operations.add(operationsItem);
    return this;
  }

   /**
   * Get operations
   * @return operations
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_OPERATIONS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public List<Operation> getOperations() {
    return operations;
  }


  @JsonProperty(JSON_PROPERTY_OPERATIONS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setOperations(List<Operation> operations) {
    this.operations = operations;
  }


  public Transaction relatedTransactions(List<RelatedTransaction> relatedTransactions) {
    this.relatedTransactions = relatedTransactions;
    return this;
  }

  public Transaction addRelatedTransactionsItem(RelatedTransaction relatedTransactionsItem) {
    if (this.relatedTransactions == null) {
      this.relatedTransactions = new ArrayList<>();
    }
    this.relatedTransactions.add(relatedTransactionsItem);
    return this;
  }

   /**
   * Get relatedTransactions
   * @return relatedTransactions
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "")
  @JsonProperty(JSON_PROPERTY_RELATED_TRANSACTIONS)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public List<RelatedTransaction> getRelatedTransactions() {
    return relatedTransactions;
  }


  @JsonProperty(JSON_PROPERTY_RELATED_TRANSACTIONS)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setRelatedTransactions(List<RelatedTransaction> relatedTransactions) {
    this.relatedTransactions = relatedTransactions;
  }


  public Transaction metadata(Object metadata) {
    this.metadata = metadata;
    return this;
  }

   /**
   * Transactions that are related to other transactions (like a cross-shard transaction) should include the tranaction_identifier of these transactions in the metadata. 
   * @return metadata
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(example = "{size=12378, lockTime=1582272577}", value = "Transactions that are related to other transactions (like a cross-shard transaction) should include the tranaction_identifier of these transactions in the metadata. ")
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
   * Return true if this Transaction object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    Transaction transaction = (Transaction) o;
    return Objects.equals(this.transactionIdentifier, transaction.transactionIdentifier) &&
        Objects.equals(this.operations, transaction.operations) &&
        Objects.equals(this.relatedTransactions, transaction.relatedTransactions) &&
        Objects.equals(this.metadata, transaction.metadata);
  }

  @Override
  public int hashCode() {
    return Objects.hash(transactionIdentifier, operations, relatedTransactions, metadata);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class Transaction {\n");
    sb.append("    transactionIdentifier: ").append(toIndentedString(transactionIdentifier)).append("\n");
    sb.append("    operations: ").append(toIndentedString(operations)).append("\n");
    sb.append("    relatedTransactions: ").append(toIndentedString(relatedTransactions)).append("\n");
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

