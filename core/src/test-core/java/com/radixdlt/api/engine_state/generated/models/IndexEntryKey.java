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
import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonSubTypes;
import com.fasterxml.jackson.annotation.JsonTypeInfo;
import com.fasterxml.jackson.annotation.JsonTypeName;
import com.fasterxml.jackson.annotation.JsonValue;
import com.radixdlt.api.engine_state.generated.models.CollectionEntryKey;
import com.radixdlt.api.engine_state.generated.models.IndexEntryKey;
import com.radixdlt.api.engine_state.generated.models.KeyValueStoreEntryKey;
import com.radixdlt.api.engine_state.generated.models.KeyValueStoreEntryKeyAllOf;
import com.radixdlt.api.engine_state.generated.models.ObjectCollectionKind;
import com.radixdlt.api.engine_state.generated.models.SborData;
import com.radixdlt.api.engine_state.generated.models.SortedIndexEntryKey;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


import com.radixdlt.api.engine_state.generated.client.JSON;
/**
 * Key within an Object&#39;s Index collection.
 */
@ApiModel(description = "Key within an Object's Index collection.")
@JsonPropertyOrder({
  IndexEntryKey.JSON_PROPERTY_KEY
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
@JsonIgnoreProperties(
  value = "kind", // ignore manually set kind, it will be automatically generated by Jackson during serialization
  allowSetters = true // allows the kind to be set during deserialization
)
@JsonTypeInfo(use = JsonTypeInfo.Id.NAME, include = JsonTypeInfo.As.PROPERTY, property = "kind", visible = true)
@JsonSubTypes({
  @JsonSubTypes.Type(value = IndexEntryKey.class, name = "Index"),
  @JsonSubTypes.Type(value = KeyValueStoreEntryKey.class, name = "KeyValueStore"),
  @JsonSubTypes.Type(value = SortedIndexEntryKey.class, name = "SortedIndex"),
})

public class IndexEntryKey extends CollectionEntryKey {
  public static final String JSON_PROPERTY_KEY = "key";
  private SborData key;

  public IndexEntryKey() { 
  }

  public IndexEntryKey key(SborData key) {
    this.key = key;
    return this;
  }

   /**
   * Get key
   * @return key
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_KEY)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public SborData getKey() {
    return key;
  }


  @JsonProperty(JSON_PROPERTY_KEY)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setKey(SborData key) {
    this.key = key;
  }


  /**
   * Return true if this IndexEntryKey object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    IndexEntryKey indexEntryKey = (IndexEntryKey) o;
    return Objects.equals(this.key, indexEntryKey.key) &&
        super.equals(o);
  }

  @Override
  public int hashCode() {
    return Objects.hash(key, super.hashCode());
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class IndexEntryKey {\n");
    sb.append("    ").append(toIndentedString(super.toString())).append("\n");
    sb.append("    key: ").append(toIndentedString(key)).append("\n");
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
  mappings.put("Index", IndexEntryKey.class);
  mappings.put("KeyValueStore", KeyValueStoreEntryKey.class);
  mappings.put("SortedIndex", SortedIndexEntryKey.class);
  mappings.put("IndexEntryKey", IndexEntryKey.class);
  JSON.registerDiscriminator(IndexEntryKey.class, "kind", mappings);
}
}

