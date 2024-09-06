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
import com.radixdlt.api.engine_state.generated.models.RequestedItemType;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * RequestedItemNotFoundDetailsAllOf
 */
@JsonPropertyOrder({
  RequestedItemNotFoundDetailsAllOf.JSON_PROPERTY_ITEM_TYPE
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class RequestedItemNotFoundDetailsAllOf {
  public static final String JSON_PROPERTY_ITEM_TYPE = "item_type";
  private RequestedItemType itemType;

  public RequestedItemNotFoundDetailsAllOf() { 
  }

  public RequestedItemNotFoundDetailsAllOf itemType(RequestedItemType itemType) {
    this.itemType = itemType;
    return this;
  }

   /**
   * Get itemType
   * @return itemType
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_ITEM_TYPE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public RequestedItemType getItemType() {
    return itemType;
  }


  @JsonProperty(JSON_PROPERTY_ITEM_TYPE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setItemType(RequestedItemType itemType) {
    this.itemType = itemType;
  }


  /**
   * Return true if this RequestedItemNotFoundDetails_allOf object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    RequestedItemNotFoundDetailsAllOf requestedItemNotFoundDetailsAllOf = (RequestedItemNotFoundDetailsAllOf) o;
    return Objects.equals(this.itemType, requestedItemNotFoundDetailsAllOf.itemType);
  }

  @Override
  public int hashCode() {
    return Objects.hash(itemType);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class RequestedItemNotFoundDetailsAllOf {\n");
    sb.append("    itemType: ").append(toIndentedString(itemType)).append("\n");
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

