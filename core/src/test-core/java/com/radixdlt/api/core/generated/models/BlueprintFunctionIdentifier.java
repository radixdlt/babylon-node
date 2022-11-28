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
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * BlueprintFunctionIdentifier
 */
@JsonPropertyOrder({
  BlueprintFunctionIdentifier.JSON_PROPERTY_PACKAGE_ADDRESS,
  BlueprintFunctionIdentifier.JSON_PROPERTY_BLUEPRINT_NAME,
  BlueprintFunctionIdentifier.JSON_PROPERTY_FUNCTION_NAME
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class BlueprintFunctionIdentifier {
  public static final String JSON_PROPERTY_PACKAGE_ADDRESS = "package_address";
  private String packageAddress;

  public static final String JSON_PROPERTY_BLUEPRINT_NAME = "blueprint_name";
  private String blueprintName;

  public static final String JSON_PROPERTY_FUNCTION_NAME = "function_name";
  private String functionName;

  public BlueprintFunctionIdentifier() { 
  }

  public BlueprintFunctionIdentifier packageAddress(String packageAddress) {
    this.packageAddress = packageAddress;
    return this;
  }

   /**
   * The Bech32m-encoded human readable version of the package address
   * @return packageAddress
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The Bech32m-encoded human readable version of the package address")
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


  public BlueprintFunctionIdentifier blueprintName(String blueprintName) {
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


  public BlueprintFunctionIdentifier functionName(String functionName) {
    this.functionName = functionName;
    return this;
  }

   /**
   * Get functionName
   * @return functionName
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_FUNCTION_NAME)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getFunctionName() {
    return functionName;
  }


  @JsonProperty(JSON_PROPERTY_FUNCTION_NAME)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setFunctionName(String functionName) {
    this.functionName = functionName;
  }


  /**
   * Return true if this BlueprintFunctionIdentifier object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    BlueprintFunctionIdentifier blueprintFunctionIdentifier = (BlueprintFunctionIdentifier) o;
    return Objects.equals(this.packageAddress, blueprintFunctionIdentifier.packageAddress) &&
        Objects.equals(this.blueprintName, blueprintFunctionIdentifier.blueprintName) &&
        Objects.equals(this.functionName, blueprintFunctionIdentifier.functionName);
  }

  @Override
  public int hashCode() {
    return Objects.hash(packageAddress, blueprintName, functionName);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class BlueprintFunctionIdentifier {\n");
    sb.append("    packageAddress: ").append(toIndentedString(packageAddress)).append("\n");
    sb.append("    blueprintName: ").append(toIndentedString(blueprintName)).append("\n");
    sb.append("    functionName: ").append(toIndentedString(functionName)).append("\n");
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

