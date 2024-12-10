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
import com.radixdlt.api.mesh.generated.models.TransactionIdentifier;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import java.util.ArrayList;
import java.util.List;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * A MempoolResponse contains all transaction identifiers in the mempool for a particular network_identifier. 
 */
@ApiModel(description = "A MempoolResponse contains all transaction identifiers in the mempool for a particular network_identifier. ")
@JsonPropertyOrder({
  MempoolResponse.JSON_PROPERTY_TRANSACTION_IDENTIFIERS
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class MempoolResponse {
  public static final String JSON_PROPERTY_TRANSACTION_IDENTIFIERS = "transaction_identifiers";
  private List<TransactionIdentifier> transactionIdentifiers = new ArrayList<>();

  public MempoolResponse() { 
  }

  public MempoolResponse transactionIdentifiers(List<TransactionIdentifier> transactionIdentifiers) {
    this.transactionIdentifiers = transactionIdentifiers;
    return this;
  }

  public MempoolResponse addTransactionIdentifiersItem(TransactionIdentifier transactionIdentifiersItem) {
    this.transactionIdentifiers.add(transactionIdentifiersItem);
    return this;
  }

   /**
   * Get transactionIdentifiers
   * @return transactionIdentifiers
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_TRANSACTION_IDENTIFIERS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public List<TransactionIdentifier> getTransactionIdentifiers() {
    return transactionIdentifiers;
  }


  @JsonProperty(JSON_PROPERTY_TRANSACTION_IDENTIFIERS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setTransactionIdentifiers(List<TransactionIdentifier> transactionIdentifiers) {
    this.transactionIdentifiers = transactionIdentifiers;
  }


  /**
   * Return true if this MempoolResponse object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    MempoolResponse mempoolResponse = (MempoolResponse) o;
    return Objects.equals(this.transactionIdentifiers, mempoolResponse.transactionIdentifiers);
  }

  @Override
  public int hashCode() {
    return Objects.hash(transactionIdentifiers);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class MempoolResponse {\n");
    sb.append("    transactionIdentifiers: ").append(toIndentedString(transactionIdentifiers)).append("\n");
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
