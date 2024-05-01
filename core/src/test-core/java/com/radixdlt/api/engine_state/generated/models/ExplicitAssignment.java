/*
 * Engine State API - Babylon (Anemone)
 * **This API is currently in Beta**  This specification may experience breaking changes as part of Babylon Node releases. Such changes will be clearly mentioned in the [babylon-node release notes](https://github.com/radixdlt/babylon-node/releases). We advise against using this API for business-critical integrations before the `version` indicated above becomes stable, which is expected in Q4 of 2024.  This API provides a complete view of the current ledger state, operating at a relatively low level (i.e. returning Entities' data and type information in a generic way, without interpreting specifics of different native or custom components).  It mirrors how the Radix Engine views the ledger state in its \"System\" layer, and thus can be useful for Scrypto developers, who need to inspect how the Engine models and stores their application's state, or how an interface / authentication scheme of another component looks like. 
 *
 * The version of the OpenAPI document: v0.1-beta
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
import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonSubTypes;
import com.fasterxml.jackson.annotation.JsonTypeInfo;
import com.fasterxml.jackson.annotation.JsonTypeName;
import com.fasterxml.jackson.annotation.JsonValue;
import com.radixdlt.api.engine_state.generated.models.AccessRule;
import com.radixdlt.api.engine_state.generated.models.Assignment;
import com.radixdlt.api.engine_state.generated.models.AssignmentType;
import com.radixdlt.api.engine_state.generated.models.ByAccessRuleBlueprintFunctionAuthorizationAllOf;
import com.radixdlt.api.engine_state.generated.models.ExplicitAssignment;
import com.radixdlt.api.engine_state.generated.models.OwnerAssignment;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


import com.radixdlt.api.engine_state.generated.client.JSON;
/**
 * Reference to a well-known Scrypto type.
 */
@ApiModel(description = "Reference to a well-known Scrypto type.")
@JsonPropertyOrder({
  ExplicitAssignment.JSON_PROPERTY_RULE
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
@JsonIgnoreProperties(
  value = "type", // ignore manually set type, it will be automatically generated by Jackson during serialization
  allowSetters = true // allows the type to be set during deserialization
)
@JsonTypeInfo(use = JsonTypeInfo.Id.NAME, include = JsonTypeInfo.As.PROPERTY, property = "type", visible = true)
@JsonSubTypes({
  @JsonSubTypes.Type(value = ExplicitAssignment.class, name = "Explicit"),
  @JsonSubTypes.Type(value = OwnerAssignment.class, name = "Owner"),
})

public class ExplicitAssignment extends Assignment {
  public static final String JSON_PROPERTY_RULE = "rule";
  private AccessRule rule;

  public ExplicitAssignment() { 
  }

  public ExplicitAssignment rule(AccessRule rule) {
    this.rule = rule;
    return this;
  }

   /**
   * Get rule
   * @return rule
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_RULE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public AccessRule getRule() {
    return rule;
  }


  @JsonProperty(JSON_PROPERTY_RULE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setRule(AccessRule rule) {
    this.rule = rule;
  }


  /**
   * Return true if this ExplicitAssignment object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    ExplicitAssignment explicitAssignment = (ExplicitAssignment) o;
    return Objects.equals(this.rule, explicitAssignment.rule) &&
        super.equals(o);
  }

  @Override
  public int hashCode() {
    return Objects.hash(rule, super.hashCode());
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class ExplicitAssignment {\n");
    sb.append("    ").append(toIndentedString(super.toString())).append("\n");
    sb.append("    rule: ").append(toIndentedString(rule)).append("\n");
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

static {
  // Initialize and register the discriminator mappings.
  Map<String, Class<?>> mappings = new HashMap<String, Class<?>>();
  mappings.put("Explicit", ExplicitAssignment.class);
  mappings.put("Owner", OwnerAssignment.class);
  mappings.put("ExplicitAssignment", ExplicitAssignment.class);
  JSON.registerDiscriminator(ExplicitAssignment.class, "type", mappings);
}
}
