/*
 * Babylon Core API
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node. It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Heavy load may impact the node's function.  If you require queries against historical ledger state, you may also wish to consider using the [Gateway API](https://betanet-gateway.redoc.ly/). 
 *
 * The version of the OpenAPI document: 0.3.0
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
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * GlobalEntityReference
 */
@JsonPropertyOrder({
  GlobalEntityReference.JSON_PROPERTY_GLOBAL_ADDRESS_HEX,
  GlobalEntityReference.JSON_PROPERTY_GLOBAL_ADDRESS,
  GlobalEntityReference.JSON_PROPERTY_ENTITY_REFERENCE
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class GlobalEntityReference {
  public static final String JSON_PROPERTY_GLOBAL_ADDRESS_HEX = "global_address_hex";
  private String globalAddressHex;

  public static final String JSON_PROPERTY_GLOBAL_ADDRESS = "global_address";
  private String globalAddress;

  public static final String JSON_PROPERTY_ENTITY_REFERENCE = "entity_reference";
  private EntityReference entityReference;

  public GlobalEntityReference() { 
  }

  public GlobalEntityReference globalAddressHex(String globalAddressHex) {
    this.globalAddressHex = globalAddressHex;
    return this;
  }

   /**
   * The hex-encoded bytes of the entity&#39;s global address
   * @return globalAddressHex
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The hex-encoded bytes of the entity's global address")
  @JsonProperty(JSON_PROPERTY_GLOBAL_ADDRESS_HEX)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getGlobalAddressHex() {
    return globalAddressHex;
  }


  @JsonProperty(JSON_PROPERTY_GLOBAL_ADDRESS_HEX)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setGlobalAddressHex(String globalAddressHex) {
    this.globalAddressHex = globalAddressHex;
  }


  public GlobalEntityReference globalAddress(String globalAddress) {
    this.globalAddress = globalAddress;
    return this;
  }

   /**
   * The Bech32m-encoded human readable version of the entity&#39;s global address
   * @return globalAddress
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The Bech32m-encoded human readable version of the entity's global address")
  @JsonProperty(JSON_PROPERTY_GLOBAL_ADDRESS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getGlobalAddress() {
    return globalAddress;
  }


  @JsonProperty(JSON_PROPERTY_GLOBAL_ADDRESS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setGlobalAddress(String globalAddress) {
    this.globalAddress = globalAddress;
  }


  public GlobalEntityReference entityReference(EntityReference entityReference) {
    this.entityReference = entityReference;
    return this;
  }

   /**
   * Get entityReference
   * @return entityReference
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_ENTITY_REFERENCE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public EntityReference getEntityReference() {
    return entityReference;
  }


  @JsonProperty(JSON_PROPERTY_ENTITY_REFERENCE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setEntityReference(EntityReference entityReference) {
    this.entityReference = entityReference;
  }


  /**
   * Return true if this GlobalEntityReference object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    GlobalEntityReference globalEntityReference = (GlobalEntityReference) o;
    return Objects.equals(this.globalAddressHex, globalEntityReference.globalAddressHex) &&
        Objects.equals(this.globalAddress, globalEntityReference.globalAddress) &&
        Objects.equals(this.entityReference, globalEntityReference.entityReference);
  }

  @Override
  public int hashCode() {
    return Objects.hash(globalAddressHex, globalAddress, entityReference);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class GlobalEntityReference {\n");
    sb.append("    globalAddressHex: ").append(toIndentedString(globalAddressHex)).append("\n");
    sb.append("    globalAddress: ").append(toIndentedString(globalAddress)).append("\n");
    sb.append("    entityReference: ").append(toIndentedString(entityReference)).append("\n");
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

