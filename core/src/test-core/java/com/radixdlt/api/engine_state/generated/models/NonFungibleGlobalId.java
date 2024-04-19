/*
 * Engine State API
 * This API provides a complete view of the current ledger state, operating at a relatively low level (i.e. returning Entities' data and type information in a generic way, without interpreting specifics of different native or custom components).  It mirrors how the Radix Engine views the ledger state in its \"System\" layer, and thus can be useful for Scrypto developers, who need to inspect how the Engine models and stores their application's state, or how an interface / authentication scheme of another component looks like. 
 *
 * The version of the OpenAPI document: v0.0.1
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
 * NonFungibleGlobalId
 */
@JsonPropertyOrder({
  NonFungibleGlobalId.JSON_PROPERTY_RESOURCE_ADDRESS,
  NonFungibleGlobalId.JSON_PROPERTY_LOCAL_ID
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class NonFungibleGlobalId {
  public static final String JSON_PROPERTY_RESOURCE_ADDRESS = "resource_address";
  private String resourceAddress;

  public static final String JSON_PROPERTY_LOCAL_ID = "local_id";
  private String localId;

  public NonFungibleGlobalId() { 
  }

  public NonFungibleGlobalId resourceAddress(String resourceAddress) {
    this.resourceAddress = resourceAddress;
    return this;
  }

   /**
   * A Bech32m-encoded human readable version of a Resource address.
   * @return resourceAddress
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "A Bech32m-encoded human readable version of a Resource address.")
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


  public NonFungibleGlobalId localId(String localId) {
    this.localId = localId;
    return this;
  }

   /**
   * A simple string representation of a non-fungible local ID, with a type-dependent formatting: * For string ids, this is &#x60;&lt;the-string-id&gt;&#x60; * For integer ids, this is &#x60;#the-integer-id#&#x60; * For bytes ids, this is &#x60;[the-lower-case-hex-representation]&#x60; * For RUID ids, this is &#x60;{...-...-...-...}&#x60; where &#x60;...&#x60; are each 16 hex characters. 
   * @return localId
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "A simple string representation of a non-fungible local ID, with a type-dependent formatting: * For string ids, this is `<the-string-id>` * For integer ids, this is `#the-integer-id#` * For bytes ids, this is `[the-lower-case-hex-representation]` * For RUID ids, this is `{...-...-...-...}` where `...` are each 16 hex characters. ")
  @JsonProperty(JSON_PROPERTY_LOCAL_ID)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getLocalId() {
    return localId;
  }


  @JsonProperty(JSON_PROPERTY_LOCAL_ID)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setLocalId(String localId) {
    this.localId = localId;
  }


  /**
   * Return true if this NonFungibleGlobalId object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    NonFungibleGlobalId nonFungibleGlobalId = (NonFungibleGlobalId) o;
    return Objects.equals(this.resourceAddress, nonFungibleGlobalId.resourceAddress) &&
        Objects.equals(this.localId, nonFungibleGlobalId.localId);
  }

  @Override
  public int hashCode() {
    return Objects.hash(resourceAddress, localId);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class NonFungibleGlobalId {\n");
    sb.append("    resourceAddress: ").append(toIndentedString(resourceAddress)).append("\n");
    sb.append("    localId: ").append(toIndentedString(localId)).append("\n");
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
