/*
 * Radix Core API - Babylon
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.0.4
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
import com.radixdlt.api.core.generated.models.FieldSubstateKey;
import com.radixdlt.api.core.generated.models.MapSubstateKey;
import com.radixdlt.api.core.generated.models.SortedSubstateKey;
import com.radixdlt.api.core.generated.models.SubstateKeyType;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


import com.radixdlt.api.core.generated.client.JSON;
/**
 * SubstateKey
 */
@JsonPropertyOrder({
  SubstateKey.JSON_PROPERTY_KEY_TYPE,
  SubstateKey.JSON_PROPERTY_DB_SORT_KEY_HEX
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
@JsonIgnoreProperties(
  value = "key_type", // ignore manually set key_type, it will be automatically generated by Jackson during serialization
  allowSetters = true // allows the key_type to be set during deserialization
)
@JsonTypeInfo(use = JsonTypeInfo.Id.NAME, include = JsonTypeInfo.As.PROPERTY, property = "key_type", visible = true)
@JsonSubTypes({
  @JsonSubTypes.Type(value = FieldSubstateKey.class, name = "Field"),
  @JsonSubTypes.Type(value = FieldSubstateKey.class, name = "FieldSubstateKey"),
  @JsonSubTypes.Type(value = MapSubstateKey.class, name = "Map"),
  @JsonSubTypes.Type(value = MapSubstateKey.class, name = "MapSubstateKey"),
  @JsonSubTypes.Type(value = SortedSubstateKey.class, name = "Sorted"),
  @JsonSubTypes.Type(value = SortedSubstateKey.class, name = "SortedSubstateKey"),
})

public class SubstateKey {
  public static final String JSON_PROPERTY_KEY_TYPE = "key_type";
  private SubstateKeyType keyType;

  public static final String JSON_PROPERTY_DB_SORT_KEY_HEX = "db_sort_key_hex";
  private String dbSortKeyHex;

  public SubstateKey() { 
  }

  public SubstateKey keyType(SubstateKeyType keyType) {
    this.keyType = keyType;
    return this;
  }

   /**
   * Get keyType
   * @return keyType
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_KEY_TYPE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public SubstateKeyType getKeyType() {
    return keyType;
  }


  @JsonProperty(JSON_PROPERTY_KEY_TYPE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setKeyType(SubstateKeyType keyType) {
    this.keyType = keyType;
  }


  public SubstateKey dbSortKeyHex(String dbSortKeyHex) {
    this.dbSortKeyHex = dbSortKeyHex;
    return this;
  }

   /**
   * The hex-encoded bytes of the partially-hashed DB sort key, under the given entity partition
   * @return dbSortKeyHex
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The hex-encoded bytes of the partially-hashed DB sort key, under the given entity partition")
  @JsonProperty(JSON_PROPERTY_DB_SORT_KEY_HEX)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getDbSortKeyHex() {
    return dbSortKeyHex;
  }


  @JsonProperty(JSON_PROPERTY_DB_SORT_KEY_HEX)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setDbSortKeyHex(String dbSortKeyHex) {
    this.dbSortKeyHex = dbSortKeyHex;
  }


  /**
   * Return true if this SubstateKey object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    SubstateKey substateKey = (SubstateKey) o;
    return Objects.equals(this.keyType, substateKey.keyType) &&
        Objects.equals(this.dbSortKeyHex, substateKey.dbSortKeyHex);
  }

  @Override
  public int hashCode() {
    return Objects.hash(keyType, dbSortKeyHex);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class SubstateKey {\n");
    sb.append("    keyType: ").append(toIndentedString(keyType)).append("\n");
    sb.append("    dbSortKeyHex: ").append(toIndentedString(dbSortKeyHex)).append("\n");
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
  mappings.put("Field", FieldSubstateKey.class);
  mappings.put("FieldSubstateKey", FieldSubstateKey.class);
  mappings.put("Map", MapSubstateKey.class);
  mappings.put("MapSubstateKey", MapSubstateKey.class);
  mappings.put("Sorted", SortedSubstateKey.class);
  mappings.put("SortedSubstateKey", SortedSubstateKey.class);
  mappings.put("SubstateKey", SubstateKey.class);
  JSON.registerDiscriminator(SubstateKey.class, "key_type", mappings);
}
}

