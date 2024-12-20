/*
 * Engine State API (Beta)
 * **This API is currently in Beta**  This specification may experience breaking changes as part of Babylon Node releases. Such changes will be clearly mentioned in the [babylon-node release notes](https://github.com/radixdlt/babylon-node/releases). We advise against using this API for business-critical integrations before the `version` indicated above becomes stable, which is expected in Q4 of 2024.  This API provides a complete view of the current ledger state, operating at a relatively low level (i.e. returning Entities' data and type information in a generic way, without interpreting specifics of different native or custom components).  It mirrors how the Radix Engine views the ledger state in its \"System\" layer, and thus can be useful for Scrypto developers, who need to inspect how the Engine models and stores their application's state, or how an interface / authentication scheme of another component looks like. 
 *
 * The version of the OpenAPI document: v1.3.0
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
 * SborData
 */
@JsonPropertyOrder({
  SborData.JSON_PROPERTY_RAW_HEX,
  SborData.JSON_PROPERTY_PROGRAMMATIC_JSON
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class SborData {
  public static final String JSON_PROPERTY_RAW_HEX = "raw_hex";
  private String rawHex;

  public static final String JSON_PROPERTY_PROGRAMMATIC_JSON = "programmatic_json";
  private Object programmaticJson;

  public SborData() { 
  }

  public SborData rawHex(String rawHex) {
    this.rawHex = rawHex;
    return this;
  }

   /**
   * Hex-encoded raw bytes (of the SBOR encoding).
   * @return rawHex
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "Hex-encoded raw bytes (of the SBOR encoding).")
  @JsonProperty(JSON_PROPERTY_RAW_HEX)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public String getRawHex() {
    return rawHex;
  }


  @JsonProperty(JSON_PROPERTY_RAW_HEX)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setRawHex(String rawHex) {
    this.rawHex = rawHex;
  }


  public SborData programmaticJson(Object programmaticJson) {
    this.programmaticJson = programmaticJson;
    return this;
  }

   /**
   * JSON representation of the SBOR structure, annotated with as much metadata (type and field names) as was available.  
   * @return programmaticJson
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "JSON representation of the SBOR structure, annotated with as much metadata (type and field names) as was available.  ")
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
    return Objects.equals(this.rawHex, sborData.rawHex) &&
        Objects.equals(this.programmaticJson, sborData.programmaticJson);
  }

  @Override
  public int hashCode() {
    return Objects.hash(rawHex, programmaticJson);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class SborData {\n");
    sb.append("    rawHex: ").append(toIndentedString(rawHex)).append("\n");
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

