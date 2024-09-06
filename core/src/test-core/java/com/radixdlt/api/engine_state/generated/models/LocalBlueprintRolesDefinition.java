/*
 * Engine State API (Beta)
 * **This API is currently in Beta**  This specification may experience breaking changes as part of Babylon Node releases. Such changes will be clearly mentioned in the [babylon-node release notes](https://github.com/radixdlt/babylon-node/releases). We advise against using this API for business-critical integrations before the `version` indicated above becomes stable, which is expected in Q4 of 2024.  This API provides a complete view of the current ledger state, operating at a relatively low level (i.e. returning Entities' data and type information in a generic way, without interpreting specifics of different native or custom components).  It mirrors how the Radix Engine views the ledger state in its \"System\" layer, and thus can be useful for Scrypto developers, who need to inspect how the Engine models and stores their application's state, or how an interface / authentication scheme of another component looks like. 
 *
 * The version of the OpenAPI document: v1.2.2
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
import com.radixdlt.api.engine_state.generated.models.BlueprintRoleInfo;
import com.radixdlt.api.engine_state.generated.models.BlueprintRolesDefinition;
import com.radixdlt.api.engine_state.generated.models.BlueprintRolesDefinitionType;
import com.radixdlt.api.engine_state.generated.models.LocalBlueprintRolesDefinition;
import com.radixdlt.api.engine_state.generated.models.LocalBlueprintRolesDefinitionAllOf;
import com.radixdlt.api.engine_state.generated.models.OuterBlueprintRolesDefinition;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import java.util.ArrayList;
import java.util.List;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


import com.radixdlt.api.engine_state.generated.client.JSON;
/**
 * LocalBlueprintRolesDefinition
 */
@JsonPropertyOrder({
  LocalBlueprintRolesDefinition.JSON_PROPERTY_DEFINITIONS
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
@JsonIgnoreProperties(
  value = "type", // ignore manually set type, it will be automatically generated by Jackson during serialization
  allowSetters = true // allows the type to be set during deserialization
)
@JsonTypeInfo(use = JsonTypeInfo.Id.NAME, include = JsonTypeInfo.As.PROPERTY, property = "type", visible = true)
@JsonSubTypes({
  @JsonSubTypes.Type(value = LocalBlueprintRolesDefinition.class, name = "Local"),
  @JsonSubTypes.Type(value = OuterBlueprintRolesDefinition.class, name = "Outer"),
})

public class LocalBlueprintRolesDefinition extends BlueprintRolesDefinition {
  public static final String JSON_PROPERTY_DEFINITIONS = "definitions";
  private List<BlueprintRoleInfo> definitions = new ArrayList<>();

  public LocalBlueprintRolesDefinition() { 
  }

  public LocalBlueprintRolesDefinition definitions(List<BlueprintRoleInfo> definitions) {
    this.definitions = definitions;
    return this;
  }

  public LocalBlueprintRolesDefinition addDefinitionsItem(BlueprintRoleInfo definitionsItem) {
    this.definitions.add(definitionsItem);
    return this;
  }

   /**
   * Get definitions
   * @return definitions
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_DEFINITIONS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public List<BlueprintRoleInfo> getDefinitions() {
    return definitions;
  }


  @JsonProperty(JSON_PROPERTY_DEFINITIONS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setDefinitions(List<BlueprintRoleInfo> definitions) {
    this.definitions = definitions;
  }


  /**
   * Return true if this LocalBlueprintRolesDefinition object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    LocalBlueprintRolesDefinition localBlueprintRolesDefinition = (LocalBlueprintRolesDefinition) o;
    return Objects.equals(this.definitions, localBlueprintRolesDefinition.definitions) &&
        super.equals(o);
  }

  @Override
  public int hashCode() {
    return Objects.hash(definitions, super.hashCode());
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class LocalBlueprintRolesDefinition {\n");
    sb.append("    ").append(toIndentedString(super.toString())).append("\n");
    sb.append("    definitions: ").append(toIndentedString(definitions)).append("\n");
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
  mappings.put("Local", LocalBlueprintRolesDefinition.class);
  mappings.put("Outer", OuterBlueprintRolesDefinition.class);
  mappings.put("LocalBlueprintRolesDefinition", LocalBlueprintRolesDefinition.class);
  JSON.registerDiscriminator(LocalBlueprintRolesDefinition.class, "type", mappings);
}
}

