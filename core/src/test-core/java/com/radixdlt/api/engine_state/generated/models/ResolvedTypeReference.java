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
import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonSubTypes;
import com.fasterxml.jackson.annotation.JsonTypeInfo;
import com.fasterxml.jackson.annotation.JsonTypeName;
import com.fasterxml.jackson.annotation.JsonValue;
import com.radixdlt.api.engine_state.generated.models.ResolvedTypeReferenceType;
import com.radixdlt.api.engine_state.generated.models.SchemaDefinedTypeReference;
import com.radixdlt.api.engine_state.generated.models.WellKnownTypeReference;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


import com.radixdlt.api.engine_state.generated.client.JSON;
/**
 * ResolvedTypeReference
 */
@JsonPropertyOrder({
  ResolvedTypeReference.JSON_PROPERTY_TYPE,
  ResolvedTypeReference.JSON_PROPERTY_NAME
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
@JsonIgnoreProperties(
  value = "type", // ignore manually set type, it will be automatically generated by Jackson during serialization
  allowSetters = true // allows the type to be set during deserialization
)
@JsonTypeInfo(use = JsonTypeInfo.Id.NAME, include = JsonTypeInfo.As.PROPERTY, property = "type", visible = true)
@JsonSubTypes({
  @JsonSubTypes.Type(value = SchemaDefinedTypeReference.class, name = "SchemaDefined"),
  @JsonSubTypes.Type(value = SchemaDefinedTypeReference.class, name = "SchemaDefinedTypeReference"),
  @JsonSubTypes.Type(value = WellKnownTypeReference.class, name = "WellKnown"),
  @JsonSubTypes.Type(value = WellKnownTypeReference.class, name = "WellKnownTypeReference"),
})

public class ResolvedTypeReference {
  public static final String JSON_PROPERTY_TYPE = "type";
  private ResolvedTypeReferenceType type;

  public static final String JSON_PROPERTY_NAME = "name";
  private String name;

  public ResolvedTypeReference() { 
  }

  public ResolvedTypeReference type(ResolvedTypeReferenceType type) {
    this.type = type;
    return this;
  }

   /**
   * Get type
   * @return type
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_TYPE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public ResolvedTypeReferenceType getType() {
    return type;
  }


  @JsonProperty(JSON_PROPERTY_TYPE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setType(ResolvedTypeReferenceType type) {
    this.type = type;
  }


  public ResolvedTypeReference name(String name) {
    this.name = name;
    return this;
  }

   /**
   * A human-readable name, derived on a best-effort basis from the type info/blueprint/schema. May be missing either because the subject deliberately has no defined name (e.g. in case of an unnamed tuple) or because the name resolution was not successful (e.g. when certain naming conventions are not observed within the relevant definitions). 
   * @return name
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "A human-readable name, derived on a best-effort basis from the type info/blueprint/schema. May be missing either because the subject deliberately has no defined name (e.g. in case of an unnamed tuple) or because the name resolution was not successful (e.g. when certain naming conventions are not observed within the relevant definitions). ")
  @JsonProperty(JSON_PROPERTY_NAME)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public String getName() {
    return name;
  }


  @JsonProperty(JSON_PROPERTY_NAME)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setName(String name) {
    this.name = name;
  }


  /**
   * Return true if this ResolvedTypeReference object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    ResolvedTypeReference resolvedTypeReference = (ResolvedTypeReference) o;
    return Objects.equals(this.type, resolvedTypeReference.type) &&
        Objects.equals(this.name, resolvedTypeReference.name);
  }

  @Override
  public int hashCode() {
    return Objects.hash(type, name);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class ResolvedTypeReference {\n");
    sb.append("    type: ").append(toIndentedString(type)).append("\n");
    sb.append("    name: ").append(toIndentedString(name)).append("\n");
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
  mappings.put("SchemaDefined", SchemaDefinedTypeReference.class);
  mappings.put("SchemaDefinedTypeReference", SchemaDefinedTypeReference.class);
  mappings.put("WellKnown", WellKnownTypeReference.class);
  mappings.put("WellKnownTypeReference", WellKnownTypeReference.class);
  mappings.put("ResolvedTypeReference", ResolvedTypeReference.class);
  JSON.registerDiscriminator(ResolvedTypeReference.class, "type", mappings);
}
}

