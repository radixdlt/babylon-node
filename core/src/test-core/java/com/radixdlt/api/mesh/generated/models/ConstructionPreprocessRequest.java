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
import com.radixdlt.api.mesh.generated.models.Operation;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import java.util.ArrayList;
import java.util.List;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * ConstructionPreprocessRequest is passed to the &#x60;/construction/preprocess&#x60; endpoint so that a Rosetta implementation can determine which metadata it needs to request for construction.  Metadata provided in this object should NEVER be a product of live data (i.e. the caller must follow some network-specific data fetching strategy outside of the Construction API to populate required Metadata). If live data is required for construction, it MUST be fetched in the call to &#x60;/construction/metadata&#x60;. 
 */
@ApiModel(description = "ConstructionPreprocessRequest is passed to the `/construction/preprocess` endpoint so that a Rosetta implementation can determine which metadata it needs to request for construction.  Metadata provided in this object should NEVER be a product of live data (i.e. the caller must follow some network-specific data fetching strategy outside of the Construction API to populate required Metadata). If live data is required for construction, it MUST be fetched in the call to `/construction/metadata`. ")
@JsonPropertyOrder({
  ConstructionPreprocessRequest.JSON_PROPERTY_NETWORK_IDENTIFIER,
  ConstructionPreprocessRequest.JSON_PROPERTY_OPERATIONS,
  ConstructionPreprocessRequest.JSON_PROPERTY_METADATA
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class ConstructionPreprocessRequest {
  public static final String JSON_PROPERTY_NETWORK_IDENTIFIER = "network_identifier";
  private NetworkIdentifier networkIdentifier;

  public static final String JSON_PROPERTY_OPERATIONS = "operations";
  private List<Operation> operations = new ArrayList<>();

  public static final String JSON_PROPERTY_METADATA = "metadata";
  private Object metadata;

  public ConstructionPreprocessRequest() { 
  }

  public ConstructionPreprocessRequest networkIdentifier(NetworkIdentifier networkIdentifier) {
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


  public ConstructionPreprocessRequest operations(List<Operation> operations) {
    this.operations = operations;
    return this;
  }

  public ConstructionPreprocessRequest addOperationsItem(Operation operationsItem) {
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


  public ConstructionPreprocessRequest metadata(Object metadata) {
    this.metadata = metadata;
    return this;
  }

   /**
   * Get metadata
   * @return metadata
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "")
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
   * Return true if this ConstructionPreprocessRequest object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    ConstructionPreprocessRequest constructionPreprocessRequest = (ConstructionPreprocessRequest) o;
    return Objects.equals(this.networkIdentifier, constructionPreprocessRequest.networkIdentifier) &&
        Objects.equals(this.operations, constructionPreprocessRequest.operations) &&
        Objects.equals(this.metadata, constructionPreprocessRequest.metadata);
  }

  @Override
  public int hashCode() {
    return Objects.hash(networkIdentifier, operations, metadata);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class ConstructionPreprocessRequest {\n");
    sb.append("    networkIdentifier: ").append(toIndentedString(networkIdentifier)).append("\n");
    sb.append("    operations: ").append(toIndentedString(operations)).append("\n");
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

