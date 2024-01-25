/*
 * Radix Core API - Babylon (Anemone)
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.1.0
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
import com.radixdlt.api.core.generated.models.EcdsaSecp256k1Signature;
import com.radixdlt.api.core.generated.models.EcdsaSecp256k1SignatureAllOf;
import com.radixdlt.api.core.generated.models.EddsaEd25519Signature;
import com.radixdlt.api.core.generated.models.PublicKeyType;
import com.radixdlt.api.core.generated.models.Signature;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


import com.radixdlt.api.core.generated.client.JSON;
/**
 * EcdsaSecp256k1Signature
 */
@JsonPropertyOrder({
  EcdsaSecp256k1Signature.JSON_PROPERTY_SIGNATURE_HEX
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
@JsonIgnoreProperties(
  value = "key_type", // ignore manually set key_type, it will be automatically generated by Jackson during serialization
  allowSetters = true // allows the key_type to be set during deserialization
)
@JsonTypeInfo(use = JsonTypeInfo.Id.NAME, include = JsonTypeInfo.As.PROPERTY, property = "key_type", visible = true)
@JsonSubTypes({
  @JsonSubTypes.Type(value = EcdsaSecp256k1Signature.class, name = "EcdsaSecp256k1"),
  @JsonSubTypes.Type(value = EddsaEd25519Signature.class, name = "EddsaEd25519"),
})

public class EcdsaSecp256k1Signature extends Signature {
  public static final String JSON_PROPERTY_SIGNATURE_HEX = "signature_hex";
  private String signatureHex;

  public EcdsaSecp256k1Signature() { 
  }

  public EcdsaSecp256k1Signature signatureHex(String signatureHex) {
    this.signatureHex = signatureHex;
    return this;
  }

   /**
   * A hex-encoded recoverable ECDSA Secp256k1 signature (65 bytes). The first byte is the recovery id, the remaining 64 bytes are the compact signature, ie &#x60;CONCAT(R, s)&#x60; where &#x60;R&#x60; and &#x60;s&#x60; are each 32-bytes in padded big-endian format.
   * @return signatureHex
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "A hex-encoded recoverable ECDSA Secp256k1 signature (65 bytes). The first byte is the recovery id, the remaining 64 bytes are the compact signature, ie `CONCAT(R, s)` where `R` and `s` are each 32-bytes in padded big-endian format.")
  @JsonProperty(JSON_PROPERTY_SIGNATURE_HEX)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getSignatureHex() {
    return signatureHex;
  }


  @JsonProperty(JSON_PROPERTY_SIGNATURE_HEX)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setSignatureHex(String signatureHex) {
    this.signatureHex = signatureHex;
  }


  /**
   * Return true if this EcdsaSecp256k1Signature object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    EcdsaSecp256k1Signature ecdsaSecp256k1Signature = (EcdsaSecp256k1Signature) o;
    return Objects.equals(this.signatureHex, ecdsaSecp256k1Signature.signatureHex) &&
        super.equals(o);
  }

  @Override
  public int hashCode() {
    return Objects.hash(signatureHex, super.hashCode());
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class EcdsaSecp256k1Signature {\n");
    sb.append("    ").append(toIndentedString(super.toString())).append("\n");
    sb.append("    signatureHex: ").append(toIndentedString(signatureHex)).append("\n");
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
  mappings.put("EcdsaSecp256k1", EcdsaSecp256k1Signature.class);
  mappings.put("EddsaEd25519", EddsaEd25519Signature.class);
  mappings.put("EcdsaSecp256k1Signature", EcdsaSecp256k1Signature.class);
  JSON.registerDiscriminator(EcdsaSecp256k1Signature.class, "key_type", mappings);
}
}

