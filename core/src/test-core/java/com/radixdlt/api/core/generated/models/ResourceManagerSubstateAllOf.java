/*
 * Babylon Core API
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node. It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Heavy load may impact the node's function.  If you require queries against historical ledger state, you may also wish to consider using the [Gateway API](https://betanet-gateway.redoc.ly/). 
 *
 * The version of the OpenAPI document: 0.2.0
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
import com.radixdlt.api.core.generated.models.EntityReference;
import com.radixdlt.api.core.generated.models.NonFungibleIdType;
import com.radixdlt.api.core.generated.models.ResourceType;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * ResourceManagerSubstateAllOf
 */
@JsonPropertyOrder({
  ResourceManagerSubstateAllOf.JSON_PROPERTY_RESOURCE_TYPE,
  ResourceManagerSubstateAllOf.JSON_PROPERTY_FUNGIBLE_DIVISIBILITY,
  ResourceManagerSubstateAllOf.JSON_PROPERTY_NON_FUNGIBLE_ID_TYPE,
  ResourceManagerSubstateAllOf.JSON_PROPERTY_TOTAL_SUPPLY,
  ResourceManagerSubstateAllOf.JSON_PROPERTY_OWNED_NON_FUNGIBLE_STORE
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class ResourceManagerSubstateAllOf {
  public static final String JSON_PROPERTY_RESOURCE_TYPE = "resource_type";
  private ResourceType resourceType;

  public static final String JSON_PROPERTY_FUNGIBLE_DIVISIBILITY = "fungible_divisibility";
  private Integer fungibleDivisibility;

  public static final String JSON_PROPERTY_NON_FUNGIBLE_ID_TYPE = "non_fungible_id_type";
  private NonFungibleIdType nonFungibleIdType;

  public static final String JSON_PROPERTY_TOTAL_SUPPLY = "total_supply";
  private String totalSupply;

  public static final String JSON_PROPERTY_OWNED_NON_FUNGIBLE_STORE = "owned_non_fungible_store";
  private EntityReference ownedNonFungibleStore;

  public ResourceManagerSubstateAllOf() { 
  }

  public ResourceManagerSubstateAllOf resourceType(ResourceType resourceType) {
    this.resourceType = resourceType;
    return this;
  }

   /**
   * Get resourceType
   * @return resourceType
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_RESOURCE_TYPE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public ResourceType getResourceType() {
    return resourceType;
  }


  @JsonProperty(JSON_PROPERTY_RESOURCE_TYPE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setResourceType(ResourceType resourceType) {
    this.resourceType = resourceType;
  }


  public ResourceManagerSubstateAllOf fungibleDivisibility(Integer fungibleDivisibility) {
    this.fungibleDivisibility = fungibleDivisibility;
    return this;
  }

   /**
   * Get fungibleDivisibility
   * minimum: 0
   * maximum: 18
   * @return fungibleDivisibility
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "")
  @JsonProperty(JSON_PROPERTY_FUNGIBLE_DIVISIBILITY)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public Integer getFungibleDivisibility() {
    return fungibleDivisibility;
  }


  @JsonProperty(JSON_PROPERTY_FUNGIBLE_DIVISIBILITY)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setFungibleDivisibility(Integer fungibleDivisibility) {
    this.fungibleDivisibility = fungibleDivisibility;
  }


  public ResourceManagerSubstateAllOf nonFungibleIdType(NonFungibleIdType nonFungibleIdType) {
    this.nonFungibleIdType = nonFungibleIdType;
    return this;
  }

   /**
   * Get nonFungibleIdType
   * @return nonFungibleIdType
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "")
  @JsonProperty(JSON_PROPERTY_NON_FUNGIBLE_ID_TYPE)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public NonFungibleIdType getNonFungibleIdType() {
    return nonFungibleIdType;
  }


  @JsonProperty(JSON_PROPERTY_NON_FUNGIBLE_ID_TYPE)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setNonFungibleIdType(NonFungibleIdType nonFungibleIdType) {
    this.nonFungibleIdType = nonFungibleIdType;
  }


  public ResourceManagerSubstateAllOf totalSupply(String totalSupply) {
    this.totalSupply = totalSupply;
    return this;
  }

   /**
   * The string-encoded decimal representing the total supply of this resource. A decimal is formed of some signed integer &#x60;m&#x60; of attos (&#x60;10^(-18)&#x60;) units, where &#x60;-2^(256 - 1) &lt;&#x3D; m &lt; 2^(256 - 1)&#x60;. 
   * @return totalSupply
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The string-encoded decimal representing the total supply of this resource. A decimal is formed of some signed integer `m` of attos (`10^(-18)`) units, where `-2^(256 - 1) <= m < 2^(256 - 1)`. ")
  @JsonProperty(JSON_PROPERTY_TOTAL_SUPPLY)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getTotalSupply() {
    return totalSupply;
  }


  @JsonProperty(JSON_PROPERTY_TOTAL_SUPPLY)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setTotalSupply(String totalSupply) {
    this.totalSupply = totalSupply;
  }


  public ResourceManagerSubstateAllOf ownedNonFungibleStore(EntityReference ownedNonFungibleStore) {
    this.ownedNonFungibleStore = ownedNonFungibleStore;
    return this;
  }

   /**
   * Get ownedNonFungibleStore
   * @return ownedNonFungibleStore
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "")
  @JsonProperty(JSON_PROPERTY_OWNED_NON_FUNGIBLE_STORE)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public EntityReference getOwnedNonFungibleStore() {
    return ownedNonFungibleStore;
  }


  @JsonProperty(JSON_PROPERTY_OWNED_NON_FUNGIBLE_STORE)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setOwnedNonFungibleStore(EntityReference ownedNonFungibleStore) {
    this.ownedNonFungibleStore = ownedNonFungibleStore;
  }


  /**
   * Return true if this ResourceManagerSubstate_allOf object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    ResourceManagerSubstateAllOf resourceManagerSubstateAllOf = (ResourceManagerSubstateAllOf) o;
    return Objects.equals(this.resourceType, resourceManagerSubstateAllOf.resourceType) &&
        Objects.equals(this.fungibleDivisibility, resourceManagerSubstateAllOf.fungibleDivisibility) &&
        Objects.equals(this.nonFungibleIdType, resourceManagerSubstateAllOf.nonFungibleIdType) &&
        Objects.equals(this.totalSupply, resourceManagerSubstateAllOf.totalSupply) &&
        Objects.equals(this.ownedNonFungibleStore, resourceManagerSubstateAllOf.ownedNonFungibleStore);
  }

  @Override
  public int hashCode() {
    return Objects.hash(resourceType, fungibleDivisibility, nonFungibleIdType, totalSupply, ownedNonFungibleStore);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class ResourceManagerSubstateAllOf {\n");
    sb.append("    resourceType: ").append(toIndentedString(resourceType)).append("\n");
    sb.append("    fungibleDivisibility: ").append(toIndentedString(fungibleDivisibility)).append("\n");
    sb.append("    nonFungibleIdType: ").append(toIndentedString(nonFungibleIdType)).append("\n");
    sb.append("    totalSupply: ").append(toIndentedString(totalSupply)).append("\n");
    sb.append("    ownedNonFungibleStore: ").append(toIndentedString(ownedNonFungibleStore)).append("\n");
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

