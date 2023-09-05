/*
 * Babylon Core API - RCnet v3.1
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  This version of the Core API belongs to the fourth release candidate of the Radix Babylon network (\"RCnet v3.1\").  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` are guaranteed to be forward compatible to Babylon mainnet launch (and beyond). We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code. 
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
import com.radixdlt.api.core.generated.models.EcdsaSecp256k1SignatureWithPublicKey;
import com.radixdlt.api.core.generated.models.EddsaEd25519PublicKey;
import com.radixdlt.api.core.generated.models.EddsaEd25519Signature;
import com.radixdlt.api.core.generated.models.EddsaEd25519SignatureWithPublicKey;
import com.radixdlt.api.core.generated.models.EddsaEd25519SignatureWithPublicKeyAllOf;
import com.radixdlt.api.core.generated.models.PublicKeyType;
import com.radixdlt.api.core.generated.models.SignatureWithPublicKey;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


import com.radixdlt.api.core.generated.client.JSON;
/**
 * EddsaEd25519SignatureWithPublicKey
 */
@JsonPropertyOrder({
  EddsaEd25519SignatureWithPublicKey.JSON_PROPERTY_PUBLIC_KEY,
  EddsaEd25519SignatureWithPublicKey.JSON_PROPERTY_SIGNATURE
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
@JsonIgnoreProperties(
  value = "key_type", // ignore manually set key_type, it will be automatically generated by Jackson during serialization
  allowSetters = true // allows the key_type to be set during deserialization
)
@JsonTypeInfo(use = JsonTypeInfo.Id.NAME, include = JsonTypeInfo.As.PROPERTY, property = "key_type", visible = true)
@JsonSubTypes({
  @JsonSubTypes.Type(value = EcdsaSecp256k1SignatureWithPublicKey.class, name = "EcdsaSecp256k1"),
  @JsonSubTypes.Type(value = EddsaEd25519SignatureWithPublicKey.class, name = "EddsaEd25519"),
})

public class EddsaEd25519SignatureWithPublicKey extends SignatureWithPublicKey {
  public static final String JSON_PROPERTY_PUBLIC_KEY = "public_key";
  private EddsaEd25519PublicKey publicKey;

  public static final String JSON_PROPERTY_SIGNATURE = "signature";
  private EddsaEd25519Signature signature;

  public EddsaEd25519SignatureWithPublicKey() { 
  }

  public EddsaEd25519SignatureWithPublicKey publicKey(EddsaEd25519PublicKey publicKey) {
    this.publicKey = publicKey;
    return this;
  }

   /**
   * Get publicKey
   * @return publicKey
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_PUBLIC_KEY)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public EddsaEd25519PublicKey getPublicKey() {
    return publicKey;
  }


  @JsonProperty(JSON_PROPERTY_PUBLIC_KEY)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setPublicKey(EddsaEd25519PublicKey publicKey) {
    this.publicKey = publicKey;
  }


  public EddsaEd25519SignatureWithPublicKey signature(EddsaEd25519Signature signature) {
    this.signature = signature;
    return this;
  }

   /**
   * Get signature
   * @return signature
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_SIGNATURE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public EddsaEd25519Signature getSignature() {
    return signature;
  }


  @JsonProperty(JSON_PROPERTY_SIGNATURE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setSignature(EddsaEd25519Signature signature) {
    this.signature = signature;
  }


  /**
   * Return true if this EddsaEd25519SignatureWithPublicKey object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    EddsaEd25519SignatureWithPublicKey eddsaEd25519SignatureWithPublicKey = (EddsaEd25519SignatureWithPublicKey) o;
    return Objects.equals(this.publicKey, eddsaEd25519SignatureWithPublicKey.publicKey) &&
        Objects.equals(this.signature, eddsaEd25519SignatureWithPublicKey.signature) &&
        super.equals(o);
  }

  @Override
  public int hashCode() {
    return Objects.hash(publicKey, signature, super.hashCode());
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class EddsaEd25519SignatureWithPublicKey {\n");
    sb.append("    ").append(toIndentedString(super.toString())).append("\n");
    sb.append("    publicKey: ").append(toIndentedString(publicKey)).append("\n");
    sb.append("    signature: ").append(toIndentedString(signature)).append("\n");
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
  mappings.put("EcdsaSecp256k1", EcdsaSecp256k1SignatureWithPublicKey.class);
  mappings.put("EddsaEd25519", EddsaEd25519SignatureWithPublicKey.class);
  mappings.put("EddsaEd25519SignatureWithPublicKey", EddsaEd25519SignatureWithPublicKey.class);
  JSON.registerDiscriminator(EddsaEd25519SignatureWithPublicKey.class, "key_type", mappings);
}
}

