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
import com.radixdlt.api.core.generated.models.EntityType;
import com.radixdlt.api.core.generated.models.NonFungibleData;
import com.radixdlt.api.core.generated.models.NonFungibleSubstateAllOf;
import com.radixdlt.api.core.generated.models.SubstateBase;
import com.radixdlt.api.core.generated.models.SubstateType;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * NonFungibleSubstate
 */
@JsonPropertyOrder({
  NonFungibleSubstate.JSON_PROPERTY_ENTITY_TYPE,
  NonFungibleSubstate.JSON_PROPERTY_SUBSTATE_TYPE,
  NonFungibleSubstate.JSON_PROPERTY_NON_FUNGIBLE_ID_HEX,
  NonFungibleSubstate.JSON_PROPERTY_IS_DELETED,
  NonFungibleSubstate.JSON_PROPERTY_NON_FUNGIBLE_DATA
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class NonFungibleSubstate {
  public static final String JSON_PROPERTY_ENTITY_TYPE = "entity_type";
  private EntityType entityType;

  public static final String JSON_PROPERTY_SUBSTATE_TYPE = "substate_type";
  private SubstateType substateType;

  public static final String JSON_PROPERTY_NON_FUNGIBLE_ID_HEX = "non_fungible_id_hex";
  private String nonFungibleIdHex;

  public static final String JSON_PROPERTY_IS_DELETED = "is_deleted";
  private Boolean isDeleted;

  public static final String JSON_PROPERTY_NON_FUNGIBLE_DATA = "non_fungible_data";
  private NonFungibleData nonFungibleData;

  public NonFungibleSubstate() { 
  }

  public NonFungibleSubstate entityType(EntityType entityType) {
    this.entityType = entityType;
    return this;
  }

   /**
   * Get entityType
   * @return entityType
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_ENTITY_TYPE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public EntityType getEntityType() {
    return entityType;
  }


  @JsonProperty(JSON_PROPERTY_ENTITY_TYPE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setEntityType(EntityType entityType) {
    this.entityType = entityType;
  }


  public NonFungibleSubstate substateType(SubstateType substateType) {
    this.substateType = substateType;
    return this;
  }

   /**
   * Get substateType
   * @return substateType
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_SUBSTATE_TYPE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public SubstateType getSubstateType() {
    return substateType;
  }


  @JsonProperty(JSON_PROPERTY_SUBSTATE_TYPE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setSubstateType(SubstateType substateType) {
    this.substateType = substateType;
  }


  public NonFungibleSubstate nonFungibleIdHex(String nonFungibleIdHex) {
    this.nonFungibleIdHex = nonFungibleIdHex;
    return this;
  }

   /**
   * The hex-encoded bytes of its non-fungible id
   * @return nonFungibleIdHex
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The hex-encoded bytes of its non-fungible id")
  @JsonProperty(JSON_PROPERTY_NON_FUNGIBLE_ID_HEX)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getNonFungibleIdHex() {
    return nonFungibleIdHex;
  }


  @JsonProperty(JSON_PROPERTY_NON_FUNGIBLE_ID_HEX)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setNonFungibleIdHex(String nonFungibleIdHex) {
    this.nonFungibleIdHex = nonFungibleIdHex;
  }


  public NonFungibleSubstate isDeleted(Boolean isDeleted) {
    this.isDeleted = isDeleted;
    return this;
  }

   /**
   * Get isDeleted
   * @return isDeleted
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_IS_DELETED)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Boolean getIsDeleted() {
    return isDeleted;
  }


  @JsonProperty(JSON_PROPERTY_IS_DELETED)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setIsDeleted(Boolean isDeleted) {
    this.isDeleted = isDeleted;
  }


  public NonFungibleSubstate nonFungibleData(NonFungibleData nonFungibleData) {
    this.nonFungibleData = nonFungibleData;
    return this;
  }

   /**
   * Get nonFungibleData
   * @return nonFungibleData
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "")
  @JsonProperty(JSON_PROPERTY_NON_FUNGIBLE_DATA)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public NonFungibleData getNonFungibleData() {
    return nonFungibleData;
  }


  @JsonProperty(JSON_PROPERTY_NON_FUNGIBLE_DATA)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setNonFungibleData(NonFungibleData nonFungibleData) {
    this.nonFungibleData = nonFungibleData;
  }


  /**
   * Return true if this NonFungibleSubstate object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    NonFungibleSubstate nonFungibleSubstate = (NonFungibleSubstate) o;
    return Objects.equals(this.entityType, nonFungibleSubstate.entityType) &&
        Objects.equals(this.substateType, nonFungibleSubstate.substateType) &&
        Objects.equals(this.nonFungibleIdHex, nonFungibleSubstate.nonFungibleIdHex) &&
        Objects.equals(this.isDeleted, nonFungibleSubstate.isDeleted) &&
        Objects.equals(this.nonFungibleData, nonFungibleSubstate.nonFungibleData);
  }

  @Override
  public int hashCode() {
    return Objects.hash(entityType, substateType, nonFungibleIdHex, isDeleted, nonFungibleData);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class NonFungibleSubstate {\n");
    sb.append("    entityType: ").append(toIndentedString(entityType)).append("\n");
    sb.append("    substateType: ").append(toIndentedString(substateType)).append("\n");
    sb.append("    nonFungibleIdHex: ").append(toIndentedString(nonFungibleIdHex)).append("\n");
    sb.append("    isDeleted: ").append(toIndentedString(isDeleted)).append("\n");
    sb.append("    nonFungibleData: ").append(toIndentedString(nonFungibleData)).append("\n");
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

