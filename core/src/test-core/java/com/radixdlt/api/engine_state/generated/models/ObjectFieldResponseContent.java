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
 * The requested field&#39;s value.
 */
@ApiModel(description = "The requested field's value.")
@JsonPropertyOrder({
  ObjectFieldResponseContent.JSON_PROPERTY_PROGRAMMATIC_JSON
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class ObjectFieldResponseContent {
  public static final String JSON_PROPERTY_PROGRAMMATIC_JSON = "programmatic_json";
  private Object programmaticJson;

  public ObjectFieldResponseContent() { 
  }

  public ObjectFieldResponseContent programmaticJson(Object programmaticJson) {
    this.programmaticJson = programmaticJson;
    return this;
  }

   /**
   * Get programmaticJson
   * @return programmaticJson
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_PROGRAMMATIC_JSON)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Object getProgrammaticJson() {
    return programmaticJson;
  }


  @JsonProperty(JSON_PROPERTY_PROGRAMMATIC_JSON)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setProgrammaticJson(Object programmaticJson) {
    this.programmaticJson = programmaticJson;
  }


  /**
   * Return true if this ObjectFieldResponse_content object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    ObjectFieldResponseContent objectFieldResponseContent = (ObjectFieldResponseContent) o;
    return Objects.equals(this.programmaticJson, objectFieldResponseContent.programmaticJson);
  }

  @Override
  public int hashCode() {
    return Objects.hash(programmaticJson);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class ObjectFieldResponseContent {\n");
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
