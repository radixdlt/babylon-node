/*
 * Babylon Core API - RCnet v3.1
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  This version of the Core API belongs to the fourth release candidate of the Radix Babylon network (\"RCnet v3.1\").  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` are guaranteed to be forward compatible to Babylon mainnet launch (and beyond). We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code. 
 *
 * The version of the OpenAPI document: 0.5.0
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
 * Various representations of an SBOR payload. Some endpoints may allow opting in/out of each representation. 
 */
@ApiModel(description = "Various representations of an SBOR payload. Some endpoints may allow opting in/out of each representation. ")
@JsonPropertyOrder({
  SborData.JSON_PROPERTY_HEX,
  SborData.JSON_PROPERTY_PROGRAMMATIC_JSON
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class SborData {
  public static final String JSON_PROPERTY_HEX = "hex";
  private String hex;

  public static final String JSON_PROPERTY_PROGRAMMATIC_JSON = "programmatic_json";
  private Object programmaticJson = null;

  public SborData() { 
  }

  public SborData hex(String hex) {
    this.hex = hex;
    return this;
  }

   /**
   * The hex-encoded, raw SBOR-encoded data
   * @return hex
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "The hex-encoded, raw SBOR-encoded data")
  @JsonProperty(JSON_PROPERTY_HEX)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public String getHex() {
    return hex;
  }


  @JsonProperty(JSON_PROPERTY_HEX)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setHex(String hex) {
    this.hex = hex;
  }


  public SborData programmaticJson(Object programmaticJson) {
    this.programmaticJson = programmaticJson;
    return this;
  }

   /**
   * The (untyped) unannotated programmatic SBOR JSON
   * @return programmaticJson
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "The (untyped) unannotated programmatic SBOR JSON")
  @JsonProperty(JSON_PROPERTY_PROGRAMMATIC_JSON)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public Object getProgrammaticJson() {
    return programmaticJson;
  }


  @JsonProperty(JSON_PROPERTY_PROGRAMMATIC_JSON)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setProgrammaticJson(Object programmaticJson) {
    this.programmaticJson = programmaticJson;
  }


  /**
   * Return true if this SborData object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    SborData sborData = (SborData) o;
    return Objects.equals(this.hex, sborData.hex) &&
        Objects.equals(this.programmaticJson, sborData.programmaticJson);
  }

  @Override
  public int hashCode() {
    return Objects.hash(hex, programmaticJson);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class SborData {\n");
    sb.append("    hex: ").append(toIndentedString(hex)).append("\n");
    sb.append("    programmaticJson: ").append(toIndentedString(programmaticJson)).append("\n");
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

