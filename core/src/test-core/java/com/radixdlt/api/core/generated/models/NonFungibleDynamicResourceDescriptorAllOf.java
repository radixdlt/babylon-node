/*
 * Babylon Core API
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 0.1.0
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
import com.radixdlt.api.core.generated.models.NonFungibleId;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * NonFungibleDynamicResourceDescriptorAllOf
 */
@JsonPropertyOrder({
  NonFungibleDynamicResourceDescriptorAllOf.JSON_PROPERTY_RESOURCE_ADDRESS,
  NonFungibleDynamicResourceDescriptorAllOf.JSON_PROPERTY_NON_FUNGIBLE_ID
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class NonFungibleDynamicResourceDescriptorAllOf {
  public static final String JSON_PROPERTY_RESOURCE_ADDRESS = "resource_address";
  private String resourceAddress;

  public static final String JSON_PROPERTY_NON_FUNGIBLE_ID = "non_fungible_id";
  private NonFungibleId nonFungibleId;

  public NonFungibleDynamicResourceDescriptorAllOf() { 
  }

  public NonFungibleDynamicResourceDescriptorAllOf resourceAddress(String resourceAddress) {
    this.resourceAddress = resourceAddress;
    return this;
  }

   /**
   * The Bech32m-encoded human readable version of the resource address
   * @return resourceAddress
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The Bech32m-encoded human readable version of the resource address")
  @JsonProperty(JSON_PROPERTY_RESOURCE_ADDRESS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getResourceAddress() {
    return resourceAddress;
  }


  @JsonProperty(JSON_PROPERTY_RESOURCE_ADDRESS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setResourceAddress(String resourceAddress) {
    this.resourceAddress = resourceAddress;
  }


  public NonFungibleDynamicResourceDescriptorAllOf nonFungibleId(NonFungibleId nonFungibleId) {
    this.nonFungibleId = nonFungibleId;
    return this;
  }

   /**
   * Get nonFungibleId
   * @return nonFungibleId
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_NON_FUNGIBLE_ID)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public NonFungibleId getNonFungibleId() {
    return nonFungibleId;
  }


  @JsonProperty(JSON_PROPERTY_NON_FUNGIBLE_ID)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setNonFungibleId(NonFungibleId nonFungibleId) {
    this.nonFungibleId = nonFungibleId;
  }


  /**
   * Return true if this NonFungibleDynamicResourceDescriptor_allOf object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    NonFungibleDynamicResourceDescriptorAllOf nonFungibleDynamicResourceDescriptorAllOf = (NonFungibleDynamicResourceDescriptorAllOf) o;
    return Objects.equals(this.resourceAddress, nonFungibleDynamicResourceDescriptorAllOf.resourceAddress) &&
        Objects.equals(this.nonFungibleId, nonFungibleDynamicResourceDescriptorAllOf.nonFungibleId);
  }

  @Override
  public int hashCode() {
    return Objects.hash(resourceAddress, nonFungibleId);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class NonFungibleDynamicResourceDescriptorAllOf {\n");
    sb.append("    resourceAddress: ").append(toIndentedString(resourceAddress)).append("\n");
    sb.append("    nonFungibleId: ").append(toIndentedString(nonFungibleId)).append("\n");
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

