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
import com.radixdlt.api.engine_state.generated.models.Assignment;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * RoleAssignmentEntry
 */
@JsonPropertyOrder({
  RoleAssignmentEntry.JSON_PROPERTY_KEY,
  RoleAssignmentEntry.JSON_PROPERTY_ASSIGNMENT
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class RoleAssignmentEntry {
  public static final String JSON_PROPERTY_KEY = "key";
  private String key;

  public static final String JSON_PROPERTY_ASSIGNMENT = "assignment";
  private Assignment assignment;

  public RoleAssignmentEntry() { 
  }

  public RoleAssignmentEntry key(String key) {
    this.key = key;
    return this;
  }

   /**
   * Role key.
   * @return key
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "Role key.")
  @JsonProperty(JSON_PROPERTY_KEY)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getKey() {
    return key;
  }


  @JsonProperty(JSON_PROPERTY_KEY)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setKey(String key) {
    this.key = key;
  }


  public RoleAssignmentEntry assignment(Assignment assignment) {
    this.assignment = assignment;
    return this;
  }

   /**
   * Get assignment
   * @return assignment
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_ASSIGNMENT)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Assignment getAssignment() {
    return assignment;
  }


  @JsonProperty(JSON_PROPERTY_ASSIGNMENT)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setAssignment(Assignment assignment) {
    this.assignment = assignment;
  }


  /**
   * Return true if this RoleAssignmentEntry object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    RoleAssignmentEntry roleAssignmentEntry = (RoleAssignmentEntry) o;
    return Objects.equals(this.key, roleAssignmentEntry.key) &&
        Objects.equals(this.assignment, roleAssignmentEntry.assignment);
  }

  @Override
  public int hashCode() {
    return Objects.hash(key, assignment);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class RoleAssignmentEntry {\n");
    sb.append("    key: ").append(toIndentedString(key)).append("\n");
    sb.append("    assignment: ").append(toIndentedString(assignment)).append("\n");
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

