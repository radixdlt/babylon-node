/*
 * Babylon Core API - RCnet v3
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  This version of the Core API belongs to the second release candidate of the Radix Babylon network (\"RCnet v3\").  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` are guaranteed to be forward compatible to Babylon mainnet launch (and beyond). We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code. 
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
import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonSubTypes;
import com.fasterxml.jackson.annotation.JsonTypeInfo;
import com.fasterxml.jackson.annotation.JsonTypeName;
import com.fasterxml.jackson.annotation.JsonValue;
import com.radixdlt.api.core.generated.models.KeyValueStoreEntryStructure;
import com.radixdlt.api.core.generated.models.KeyValueStoreEntryStructureAllOf;
import com.radixdlt.api.core.generated.models.LocalTypeIndex;
import com.radixdlt.api.core.generated.models.ObjectFieldStructure;
import com.radixdlt.api.core.generated.models.ObjectIndexPartitionEntryStructure;
import com.radixdlt.api.core.generated.models.ObjectKeyValuePartitionEntryStructure;
import com.radixdlt.api.core.generated.models.ObjectSortedIndexPartitionEntryStructure;
import com.radixdlt.api.core.generated.models.SubstateSystemStructure;
import com.radixdlt.api.core.generated.models.SubstateSystemStructureType;
import com.radixdlt.api.core.generated.models.SystemFieldStructure;
import com.radixdlt.api.core.generated.models.SystemSchemaStructure;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


import com.radixdlt.api.core.generated.client.JSON;
/**
 * KeyValueStoreEntryStructure
 */
@JsonPropertyOrder({
  KeyValueStoreEntryStructure.JSON_PROPERTY_KEY_VALUE_STORE_ADDRESS,
  KeyValueStoreEntryStructure.JSON_PROPERTY_KEY_SCHEMA_HASH,
  KeyValueStoreEntryStructure.JSON_PROPERTY_KEY_LOCAL_TYPE_INDEX,
  KeyValueStoreEntryStructure.JSON_PROPERTY_VALUE_SCHEMA_HASH,
  KeyValueStoreEntryStructure.JSON_PROPERTY_VALUE_LOCAL_TYPE_INDEX
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
@JsonIgnoreProperties(
  value = "type", // ignore manually set type, it will be automatically generated by Jackson during serialization
  allowSetters = true // allows the type to be set during deserialization
)
@JsonTypeInfo(use = JsonTypeInfo.Id.NAME, include = JsonTypeInfo.As.PROPERTY, property = "type", visible = true)
@JsonSubTypes({
  @JsonSubTypes.Type(value = KeyValueStoreEntryStructure.class, name = "KeyValueStoreEntry"),
  @JsonSubTypes.Type(value = ObjectFieldStructure.class, name = "ObjectField"),
  @JsonSubTypes.Type(value = ObjectIndexPartitionEntryStructure.class, name = "ObjectIndexPartitionEntry"),
  @JsonSubTypes.Type(value = ObjectKeyValuePartitionEntryStructure.class, name = "ObjectKeyValuePartitionEntry"),
  @JsonSubTypes.Type(value = ObjectSortedIndexPartitionEntryStructure.class, name = "ObjectSortedIndexPartitionEntry"),
  @JsonSubTypes.Type(value = SystemFieldStructure.class, name = "SystemField"),
  @JsonSubTypes.Type(value = SystemSchemaStructure.class, name = "SystemSchema"),
})

public class KeyValueStoreEntryStructure extends SubstateSystemStructure {
  public static final String JSON_PROPERTY_KEY_VALUE_STORE_ADDRESS = "key_value_store_address";
  private String keyValueStoreAddress;

  public static final String JSON_PROPERTY_KEY_SCHEMA_HASH = "key_schema_hash";
  private String keySchemaHash;

  public static final String JSON_PROPERTY_KEY_LOCAL_TYPE_INDEX = "key_local_type_index";
  private LocalTypeIndex keyLocalTypeIndex;

  public static final String JSON_PROPERTY_VALUE_SCHEMA_HASH = "value_schema_hash";
  private String valueSchemaHash;

  public static final String JSON_PROPERTY_VALUE_LOCAL_TYPE_INDEX = "value_local_type_index";
  private LocalTypeIndex valueLocalTypeIndex;

  public KeyValueStoreEntryStructure() { 
  }

  public KeyValueStoreEntryStructure keyValueStoreAddress(String keyValueStoreAddress) {
    this.keyValueStoreAddress = keyValueStoreAddress;
    return this;
  }

   /**
   * Bech32m-encoded human readable version of the entity&#39;s address (ie the entity&#39;s node id)
   * @return keyValueStoreAddress
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "Bech32m-encoded human readable version of the entity's address (ie the entity's node id)")
  @JsonProperty(JSON_PROPERTY_KEY_VALUE_STORE_ADDRESS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getKeyValueStoreAddress() {
    return keyValueStoreAddress;
  }


  @JsonProperty(JSON_PROPERTY_KEY_VALUE_STORE_ADDRESS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setKeyValueStoreAddress(String keyValueStoreAddress) {
    this.keyValueStoreAddress = keyValueStoreAddress;
  }


  public KeyValueStoreEntryStructure keySchemaHash(String keySchemaHash) {
    this.keySchemaHash = keySchemaHash;
    return this;
  }

   /**
   * The hex-encoded schema hash, capturing the identity of an SBOR schema.
   * @return keySchemaHash
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The hex-encoded schema hash, capturing the identity of an SBOR schema.")
  @JsonProperty(JSON_PROPERTY_KEY_SCHEMA_HASH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getKeySchemaHash() {
    return keySchemaHash;
  }


  @JsonProperty(JSON_PROPERTY_KEY_SCHEMA_HASH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setKeySchemaHash(String keySchemaHash) {
    this.keySchemaHash = keySchemaHash;
  }


  public KeyValueStoreEntryStructure keyLocalTypeIndex(LocalTypeIndex keyLocalTypeIndex) {
    this.keyLocalTypeIndex = keyLocalTypeIndex;
    return this;
  }

   /**
   * Get keyLocalTypeIndex
   * @return keyLocalTypeIndex
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_KEY_LOCAL_TYPE_INDEX)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public LocalTypeIndex getKeyLocalTypeIndex() {
    return keyLocalTypeIndex;
  }


  @JsonProperty(JSON_PROPERTY_KEY_LOCAL_TYPE_INDEX)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setKeyLocalTypeIndex(LocalTypeIndex keyLocalTypeIndex) {
    this.keyLocalTypeIndex = keyLocalTypeIndex;
  }


  public KeyValueStoreEntryStructure valueSchemaHash(String valueSchemaHash) {
    this.valueSchemaHash = valueSchemaHash;
    return this;
  }

   /**
   * The hex-encoded schema hash, capturing the identity of an SBOR schema.
   * @return valueSchemaHash
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The hex-encoded schema hash, capturing the identity of an SBOR schema.")
  @JsonProperty(JSON_PROPERTY_VALUE_SCHEMA_HASH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getValueSchemaHash() {
    return valueSchemaHash;
  }


  @JsonProperty(JSON_PROPERTY_VALUE_SCHEMA_HASH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setValueSchemaHash(String valueSchemaHash) {
    this.valueSchemaHash = valueSchemaHash;
  }


  public KeyValueStoreEntryStructure valueLocalTypeIndex(LocalTypeIndex valueLocalTypeIndex) {
    this.valueLocalTypeIndex = valueLocalTypeIndex;
    return this;
  }

   /**
   * Get valueLocalTypeIndex
   * @return valueLocalTypeIndex
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_VALUE_LOCAL_TYPE_INDEX)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public LocalTypeIndex getValueLocalTypeIndex() {
    return valueLocalTypeIndex;
  }


  @JsonProperty(JSON_PROPERTY_VALUE_LOCAL_TYPE_INDEX)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setValueLocalTypeIndex(LocalTypeIndex valueLocalTypeIndex) {
    this.valueLocalTypeIndex = valueLocalTypeIndex;
  }


  /**
   * Return true if this KeyValueStoreEntryStructure object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    KeyValueStoreEntryStructure keyValueStoreEntryStructure = (KeyValueStoreEntryStructure) o;
    return Objects.equals(this.keyValueStoreAddress, keyValueStoreEntryStructure.keyValueStoreAddress) &&
        Objects.equals(this.keySchemaHash, keyValueStoreEntryStructure.keySchemaHash) &&
        Objects.equals(this.keyLocalTypeIndex, keyValueStoreEntryStructure.keyLocalTypeIndex) &&
        Objects.equals(this.valueSchemaHash, keyValueStoreEntryStructure.valueSchemaHash) &&
        Objects.equals(this.valueLocalTypeIndex, keyValueStoreEntryStructure.valueLocalTypeIndex) &&
        super.equals(o);
  }

  @Override
  public int hashCode() {
    return Objects.hash(keyValueStoreAddress, keySchemaHash, keyLocalTypeIndex, valueSchemaHash, valueLocalTypeIndex, super.hashCode());
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class KeyValueStoreEntryStructure {\n");
    sb.append("    ").append(toIndentedString(super.toString())).append("\n");
    sb.append("    keyValueStoreAddress: ").append(toIndentedString(keyValueStoreAddress)).append("\n");
    sb.append("    keySchemaHash: ").append(toIndentedString(keySchemaHash)).append("\n");
    sb.append("    keyLocalTypeIndex: ").append(toIndentedString(keyLocalTypeIndex)).append("\n");
    sb.append("    valueSchemaHash: ").append(toIndentedString(valueSchemaHash)).append("\n");
    sb.append("    valueLocalTypeIndex: ").append(toIndentedString(valueLocalTypeIndex)).append("\n");
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
  mappings.put("KeyValueStoreEntry", KeyValueStoreEntryStructure.class);
  mappings.put("ObjectField", ObjectFieldStructure.class);
  mappings.put("ObjectIndexPartitionEntry", ObjectIndexPartitionEntryStructure.class);
  mappings.put("ObjectKeyValuePartitionEntry", ObjectKeyValuePartitionEntryStructure.class);
  mappings.put("ObjectSortedIndexPartitionEntry", ObjectSortedIndexPartitionEntryStructure.class);
  mappings.put("SystemField", SystemFieldStructure.class);
  mappings.put("SystemSchema", SystemSchemaStructure.class);
  mappings.put("KeyValueStoreEntryStructure", KeyValueStoreEntryStructure.class);
  JSON.registerDiscriminator(KeyValueStoreEntryStructure.class, "type", mappings);
}
}
