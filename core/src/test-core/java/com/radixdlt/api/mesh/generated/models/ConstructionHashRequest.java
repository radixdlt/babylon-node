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
import com.radixdlt.api.mesh.generated.models.NetworkIdentifier;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * ConstructionHashRequest is the input to the &#x60;/construction/hash&#x60; endpoint. 
 */
@ApiModel(description = "ConstructionHashRequest is the input to the `/construction/hash` endpoint. ")
@JsonPropertyOrder({
  ConstructionHashRequest.JSON_PROPERTY_NETWORK_IDENTIFIER,
  ConstructionHashRequest.JSON_PROPERTY_SIGNED_TRANSACTION
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class ConstructionHashRequest {
  public static final String JSON_PROPERTY_NETWORK_IDENTIFIER = "network_identifier";
  private NetworkIdentifier networkIdentifier;

  public static final String JSON_PROPERTY_SIGNED_TRANSACTION = "signed_transaction";
  private String signedTransaction;

  public ConstructionHashRequest() { 
  }

  public ConstructionHashRequest networkIdentifier(NetworkIdentifier networkIdentifier) {
    this.networkIdentifier = networkIdentifier;
    return this;
  }

   /**
   * Get networkIdentifier
   * @return networkIdentifier
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_NETWORK_IDENTIFIER)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public NetworkIdentifier getNetworkIdentifier() {
    return networkIdentifier;
  }


  @JsonProperty(JSON_PROPERTY_NETWORK_IDENTIFIER)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setNetworkIdentifier(NetworkIdentifier networkIdentifier) {
    this.networkIdentifier = networkIdentifier;
  }


  public ConstructionHashRequest signedTransaction(String signedTransaction) {
    this.signedTransaction = signedTransaction;
    return this;
  }

   /**
   * Get signedTransaction
   * @return signedTransaction
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_SIGNED_TRANSACTION)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getSignedTransaction() {
    return signedTransaction;
  }


  @JsonProperty(JSON_PROPERTY_SIGNED_TRANSACTION)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setSignedTransaction(String signedTransaction) {
    this.signedTransaction = signedTransaction;
  }


  /**
   * Return true if this ConstructionHashRequest object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    ConstructionHashRequest constructionHashRequest = (ConstructionHashRequest) o;
    return Objects.equals(this.networkIdentifier, constructionHashRequest.networkIdentifier) &&
        Objects.equals(this.signedTransaction, constructionHashRequest.signedTransaction);
  }

  @Override
  public int hashCode() {
    return Objects.hash(networkIdentifier, signedTransaction);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class ConstructionHashRequest {\n");
    sb.append("    networkIdentifier: ").append(toIndentedString(networkIdentifier)).append("\n");
    sb.append("    signedTransaction: ").append(toIndentedString(signedTransaction)).append("\n");
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

