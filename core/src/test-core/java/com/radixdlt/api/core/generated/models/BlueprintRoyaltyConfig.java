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
import com.radixdlt.api.core.generated.models.RoyaltyConfig;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * BlueprintRoyaltyConfig
 */
@JsonPropertyOrder({
  BlueprintRoyaltyConfig.JSON_PROPERTY_BLUEPRINT_NAME,
  BlueprintRoyaltyConfig.JSON_PROPERTY_ROYALTY_CONFIG
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class BlueprintRoyaltyConfig {
  public static final String JSON_PROPERTY_BLUEPRINT_NAME = "blueprint_name";
  private String blueprintName;

  public static final String JSON_PROPERTY_ROYALTY_CONFIG = "royalty_config";
  private RoyaltyConfig royaltyConfig;

  public BlueprintRoyaltyConfig() { 
  }

  public BlueprintRoyaltyConfig blueprintName(String blueprintName) {
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


  public BlueprintRoyaltyConfig royaltyConfig(RoyaltyConfig royaltyConfig) {
    this.royaltyConfig = royaltyConfig;
    return this;
  }

   /**
   * Get royaltyConfig
   * @return royaltyConfig
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_ROYALTY_CONFIG)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public RoyaltyConfig getRoyaltyConfig() {
    return royaltyConfig;
  }


  @JsonProperty(JSON_PROPERTY_ROYALTY_CONFIG)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setRoyaltyConfig(RoyaltyConfig royaltyConfig) {
    this.royaltyConfig = royaltyConfig;
  }


  /**
   * Return true if this BlueprintRoyaltyConfig object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    BlueprintRoyaltyConfig blueprintRoyaltyConfig = (BlueprintRoyaltyConfig) o;
    return Objects.equals(this.blueprintName, blueprintRoyaltyConfig.blueprintName) &&
        Objects.equals(this.royaltyConfig, blueprintRoyaltyConfig.royaltyConfig);
  }

  @Override
  public int hashCode() {
    return Objects.hash(blueprintName, royaltyConfig);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class BlueprintRoyaltyConfig {\n");
    sb.append("    blueprintName: ").append(toIndentedString(blueprintName)).append("\n");
    sb.append("    royaltyConfig: ").append(toIndentedString(royaltyConfig)).append("\n");
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

