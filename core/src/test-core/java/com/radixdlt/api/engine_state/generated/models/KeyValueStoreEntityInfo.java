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
import com.radixdlt.api.engine_state.generated.models.EntityAncestryInfo;
import com.radixdlt.api.engine_state.generated.models.EntityInfo;
import com.radixdlt.api.engine_state.generated.models.KeyValueStoreEntityInfo;
import com.radixdlt.api.engine_state.generated.models.KeyValueStoreEntityInfoAllOf;
import com.radixdlt.api.engine_state.generated.models.ObjectEntityInfo;
import com.radixdlt.api.engine_state.generated.models.ResolvedTypeReference;
import com.radixdlt.api.engine_state.generated.models.SystemType;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


import com.radixdlt.api.engine_state.generated.client.JSON;
/**
 * KeyValueStoreEntityInfo
 */
@JsonPropertyOrder({
  KeyValueStoreEntityInfo.JSON_PROPERTY_KEY_TYPE_REFERENCE,
  KeyValueStoreEntityInfo.JSON_PROPERTY_VALUE_TYPE_REFERENCE
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
@JsonIgnoreProperties(
  value = "system_type", // ignore manually set system_type, it will be automatically generated by Jackson during serialization
  allowSetters = true // allows the system_type to be set during deserialization
)
@JsonTypeInfo(use = JsonTypeInfo.Id.NAME, include = JsonTypeInfo.As.PROPERTY, property = "system_type", visible = true)
@JsonSubTypes({
  @JsonSubTypes.Type(value = KeyValueStoreEntityInfo.class, name = "KeyValueStore"),
  @JsonSubTypes.Type(value = ObjectEntityInfo.class, name = "Object"),
})

public class KeyValueStoreEntityInfo extends EntityInfo {
  public static final String JSON_PROPERTY_KEY_TYPE_REFERENCE = "key_type_reference";
  private ResolvedTypeReference keyTypeReference;

  public static final String JSON_PROPERTY_VALUE_TYPE_REFERENCE = "value_type_reference";
  private ResolvedTypeReference valueTypeReference;

  public KeyValueStoreEntityInfo() { 
  }

  public KeyValueStoreEntityInfo keyTypeReference(ResolvedTypeReference keyTypeReference) {
    this.keyTypeReference = keyTypeReference;
    return this;
  }

   /**
   * Get keyTypeReference
   * @return keyTypeReference
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_KEY_TYPE_REFERENCE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public ResolvedTypeReference getKeyTypeReference() {
    return keyTypeReference;
  }


  @JsonProperty(JSON_PROPERTY_KEY_TYPE_REFERENCE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setKeyTypeReference(ResolvedTypeReference keyTypeReference) {
    this.keyTypeReference = keyTypeReference;
  }


  public KeyValueStoreEntityInfo valueTypeReference(ResolvedTypeReference valueTypeReference) {
    this.valueTypeReference = valueTypeReference;
    return this;
  }

   /**
   * Get valueTypeReference
   * @return valueTypeReference
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_VALUE_TYPE_REFERENCE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public ResolvedTypeReference getValueTypeReference() {
    return valueTypeReference;
  }


  @JsonProperty(JSON_PROPERTY_VALUE_TYPE_REFERENCE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setValueTypeReference(ResolvedTypeReference valueTypeReference) {
    this.valueTypeReference = valueTypeReference;
  }


  /**
   * Return true if this KeyValueStoreEntityInfo object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    KeyValueStoreEntityInfo keyValueStoreEntityInfo = (KeyValueStoreEntityInfo) o;
    return Objects.equals(this.keyTypeReference, keyValueStoreEntityInfo.keyTypeReference) &&
        Objects.equals(this.valueTypeReference, keyValueStoreEntityInfo.valueTypeReference) &&
        super.equals(o);
  }

  @Override
  public int hashCode() {
    return Objects.hash(keyTypeReference, valueTypeReference, super.hashCode());
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class KeyValueStoreEntityInfo {\n");
    sb.append("    ").append(toIndentedString(super.toString())).append("\n");
    sb.append("    keyTypeReference: ").append(toIndentedString(keyTypeReference)).append("\n");
    sb.append("    valueTypeReference: ").append(toIndentedString(valueTypeReference)).append("\n");
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
  mappings.put("KeyValueStore", KeyValueStoreEntityInfo.class);
  mappings.put("Object", ObjectEntityInfo.class);
  mappings.put("KeyValueStoreEntityInfo", KeyValueStoreEntityInfo.class);
  JSON.registerDiscriminator(KeyValueStoreEntityInfo.class, "system_type", mappings);
}
}
