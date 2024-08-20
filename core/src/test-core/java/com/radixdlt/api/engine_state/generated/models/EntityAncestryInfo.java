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
 * Information about the ancestor entities - i.e. the immediate parent, and the root entity. Only present when the subject entity is not a root entity itself. 
 */
@ApiModel(description = "Information about the ancestor entities - i.e. the immediate parent, and the root entity. Only present when the subject entity is not a root entity itself. ")
@JsonPropertyOrder({
  EntityAncestryInfo.JSON_PROPERTY_PARENT_ENTITY_ADDRESS,
  EntityAncestryInfo.JSON_PROPERTY_ROOT_ENTITY_ADDRESS
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class EntityAncestryInfo {
  public static final String JSON_PROPERTY_PARENT_ENTITY_ADDRESS = "parent_entity_address";
  private String parentEntityAddress;

  public static final String JSON_PROPERTY_ROOT_ENTITY_ADDRESS = "root_entity_address";
  private String rootEntityAddress;

  public EntityAncestryInfo() { 
  }

  public EntityAncestryInfo parentEntityAddress(String parentEntityAddress) {
    this.parentEntityAddress = parentEntityAddress;
    return this;
  }

   /**
   * A Bech32m-encoded, human readable rendering of an arbitrary Entity&#39;s address.
   * @return parentEntityAddress
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "A Bech32m-encoded, human readable rendering of an arbitrary Entity's address.")
  @JsonProperty(JSON_PROPERTY_PARENT_ENTITY_ADDRESS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getParentEntityAddress() {
    return parentEntityAddress;
  }


  @JsonProperty(JSON_PROPERTY_PARENT_ENTITY_ADDRESS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setParentEntityAddress(String parentEntityAddress) {
    this.parentEntityAddress = parentEntityAddress;
  }


  public EntityAncestryInfo rootEntityAddress(String rootEntityAddress) {
    this.rootEntityAddress = rootEntityAddress;
    return this;
  }

   /**
   * A Bech32m-encoded, human readable rendering of any global Entity&#39;s address.
   * @return rootEntityAddress
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "A Bech32m-encoded, human readable rendering of any global Entity's address.")
  @JsonProperty(JSON_PROPERTY_ROOT_ENTITY_ADDRESS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getRootEntityAddress() {
    return rootEntityAddress;
  }


  @JsonProperty(JSON_PROPERTY_ROOT_ENTITY_ADDRESS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setRootEntityAddress(String rootEntityAddress) {
    this.rootEntityAddress = rootEntityAddress;
  }


  /**
   * Return true if this EntityAncestryInfo object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    EntityAncestryInfo entityAncestryInfo = (EntityAncestryInfo) o;
    return Objects.equals(this.parentEntityAddress, entityAncestryInfo.parentEntityAddress) &&
        Objects.equals(this.rootEntityAddress, entityAncestryInfo.rootEntityAddress);
  }

  @Override
  public int hashCode() {
    return Objects.hash(parentEntityAddress, rootEntityAddress);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class EntityAncestryInfo {\n");
    sb.append("    parentEntityAddress: ").append(toIndentedString(parentEntityAddress)).append("\n");
    sb.append("    rootEntityAddress: ").append(toIndentedString(rootEntityAddress)).append("\n");
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

