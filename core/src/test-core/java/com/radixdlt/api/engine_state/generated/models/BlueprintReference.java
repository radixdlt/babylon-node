/*
 * Engine State API - Babylon (Anemone)
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
 * BlueprintReference
 */
@JsonPropertyOrder({
  BlueprintReference.JSON_PROPERTY_PACKAGE_ADDRESS,
  BlueprintReference.JSON_PROPERTY_BLUEPRINT_NAME,
  BlueprintReference.JSON_PROPERTY_BLUEPRINT_VERSION
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class BlueprintReference {
  public static final String JSON_PROPERTY_PACKAGE_ADDRESS = "package_address";
  private String packageAddress;

  public static final String JSON_PROPERTY_BLUEPRINT_NAME = "blueprint_name";
  private String blueprintName;

  public static final String JSON_PROPERTY_BLUEPRINT_VERSION = "blueprint_version";
  private String blueprintVersion;

  public BlueprintReference() { 
  }

  public BlueprintReference packageAddress(String packageAddress) {
    this.packageAddress = packageAddress;
    return this;
  }

   /**
   * A Bech32m-encoded, human readable rendering of a Package address.
   * @return packageAddress
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "A Bech32m-encoded, human readable rendering of a Package address.")
  @JsonProperty(JSON_PROPERTY_PACKAGE_ADDRESS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getPackageAddress() {
    return packageAddress;
  }


  @JsonProperty(JSON_PROPERTY_PACKAGE_ADDRESS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setPackageAddress(String packageAddress) {
    this.packageAddress = packageAddress;
  }


  public BlueprintReference blueprintName(String blueprintName) {
    this.blueprintName = blueprintName;
    return this;
  }

   /**
   * Get blueprintName
   * @return blueprintName
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_BLUEPRINT_NAME)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getBlueprintName() {
    return blueprintName;
  }


  @JsonProperty(JSON_PROPERTY_BLUEPRINT_NAME)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setBlueprintName(String blueprintName) {
    this.blueprintName = blueprintName;
  }


  public BlueprintReference blueprintVersion(String blueprintVersion) {
    this.blueprintVersion = blueprintVersion;
    return this;
  }

   /**
   * A string of format &#x60;Major.Minor.Patch&#x60; (all parts being &#x60;u32&#x60;).
   * @return blueprintVersion
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "A string of format `Major.Minor.Patch` (all parts being `u32`).")
  @JsonProperty(JSON_PROPERTY_BLUEPRINT_VERSION)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getBlueprintVersion() {
    return blueprintVersion;
  }


  @JsonProperty(JSON_PROPERTY_BLUEPRINT_VERSION)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setBlueprintVersion(String blueprintVersion) {
    this.blueprintVersion = blueprintVersion;
  }


  /**
   * Return true if this BlueprintReference object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    BlueprintReference blueprintReference = (BlueprintReference) o;
    return Objects.equals(this.packageAddress, blueprintReference.packageAddress) &&
        Objects.equals(this.blueprintName, blueprintReference.blueprintName) &&
        Objects.equals(this.blueprintVersion, blueprintReference.blueprintVersion);
  }

  @Override
  public int hashCode() {
    return Objects.hash(packageAddress, blueprintName, blueprintVersion);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class BlueprintReference {\n");
    sb.append("    packageAddress: ").append(toIndentedString(packageAddress)).append("\n");
    sb.append("    blueprintName: ").append(toIndentedString(blueprintName)).append("\n");
    sb.append("    blueprintVersion: ").append(toIndentedString(blueprintVersion)).append("\n");
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

