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
import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonSubTypes;
import com.fasterxml.jackson.annotation.JsonTypeInfo;
import com.fasterxml.jackson.annotation.JsonTypeName;
import com.fasterxml.jackson.annotation.JsonValue;
import com.radixdlt.api.engine_state.generated.models.EcdsaSecp256k1PublicKey;
import com.radixdlt.api.engine_state.generated.models.EddsaEd25519PublicKey;
import com.radixdlt.api.engine_state.generated.models.EddsaEd25519PublicKeyAllOf;
import com.radixdlt.api.engine_state.generated.models.PublicKey;
import com.radixdlt.api.engine_state.generated.models.PublicKeyType;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


import com.radixdlt.api.engine_state.generated.client.JSON;
/**
 * EddsaEd25519PublicKey
 */
@JsonPropertyOrder({
  EddsaEd25519PublicKey.JSON_PROPERTY_KEY_HEX
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
@JsonIgnoreProperties(
  value = "key_type", // ignore manually set key_type, it will be automatically generated by Jackson during serialization
  allowSetters = true // allows the key_type to be set during deserialization
)
@JsonTypeInfo(use = JsonTypeInfo.Id.NAME, include = JsonTypeInfo.As.PROPERTY, property = "key_type", visible = true)
@JsonSubTypes({
  @JsonSubTypes.Type(value = EcdsaSecp256k1PublicKey.class, name = "EcdsaSecp256k1"),
  @JsonSubTypes.Type(value = EddsaEd25519PublicKey.class, name = "EddsaEd25519"),
})

public class EddsaEd25519PublicKey extends PublicKey {
  public static final String JSON_PROPERTY_KEY_HEX = "key_hex";
  private String keyHex;

  public EddsaEd25519PublicKey() { 
  }

  public EddsaEd25519PublicKey keyHex(String keyHex) {
    this.keyHex = keyHex;
    return this;
  }

   /**
   * The hex-encoded compressed EdDSA Ed25519 public key (32 bytes)
   * @return keyHex
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The hex-encoded compressed EdDSA Ed25519 public key (32 bytes)")
  @JsonProperty(JSON_PROPERTY_KEY_HEX)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getKeyHex() {
    return keyHex;
  }


  @JsonProperty(JSON_PROPERTY_KEY_HEX)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setKeyHex(String keyHex) {
    this.keyHex = keyHex;
  }


  /**
   * Return true if this EddsaEd25519PublicKey object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    EddsaEd25519PublicKey eddsaEd25519PublicKey = (EddsaEd25519PublicKey) o;
    return Objects.equals(this.keyHex, eddsaEd25519PublicKey.keyHex) &&
        super.equals(o);
  }

  @Override
  public int hashCode() {
    return Objects.hash(keyHex, super.hashCode());
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class EddsaEd25519PublicKey {\n");
    sb.append("    ").append(toIndentedString(super.toString())).append("\n");
    sb.append("    keyHex: ").append(toIndentedString(keyHex)).append("\n");
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
  mappings.put("EcdsaSecp256k1", EcdsaSecp256k1PublicKey.class);
  mappings.put("EddsaEd25519", EddsaEd25519PublicKey.class);
  mappings.put("EddsaEd25519PublicKey", EddsaEd25519PublicKey.class);
  JSON.registerDiscriminator(EddsaEd25519PublicKey.class, "key_type", mappings);
}
}

