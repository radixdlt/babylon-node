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
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonTypeName;
import com.fasterxml.jackson.annotation.JsonValue;
import com.radixdlt.api.core.generated.models.EncryptedMessageDecryptor;
import com.radixdlt.api.core.generated.models.PublicKey;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import java.util.ArrayList;
import java.util.List;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * A decryptor set for a particular ECDSA curve type. The (128-bit) AES-GCM symmetric key is encrypted separately for each decryptor public key via (256-bit) AES-KeyWrap. AES-KeyWrap uses a key derived via a KDF (Key Derivation Function) using a shared secret. For each decryptor public key, we create a shared curve point &#x60;G&#x60; via static Diffie-Helman between the decryptor public key, and a per-transaction ephemeral public key for that curve type. We then use that shared secret with a key derivation function to create the (256-bit) KEK (Key Encrypting Key): &#x60;KEK &#x3D; HKDF(hash: Blake2b, secret: x co-ord of G, salt: [], length: 256 bits)&#x60;. 
 */
@ApiModel(description = "A decryptor set for a particular ECDSA curve type. The (128-bit) AES-GCM symmetric key is encrypted separately for each decryptor public key via (256-bit) AES-KeyWrap. AES-KeyWrap uses a key derived via a KDF (Key Derivation Function) using a shared secret. For each decryptor public key, we create a shared curve point `G` via static Diffie-Helman between the decryptor public key, and a per-transaction ephemeral public key for that curve type. We then use that shared secret with a key derivation function to create the (256-bit) KEK (Key Encrypting Key): `KEK = HKDF(hash: Blake2b, secret: x co-ord of G, salt: [], length: 256 bits)`. ")
@JsonPropertyOrder({
  EncryptedMessageCurveDecryptorSet.JSON_PROPERTY_DH_EPHEMERAL_PUBLIC_KEY,
  EncryptedMessageCurveDecryptorSet.JSON_PROPERTY_DECRYPTORS
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class EncryptedMessageCurveDecryptorSet {
  public static final String JSON_PROPERTY_DH_EPHEMERAL_PUBLIC_KEY = "dh_ephemeral_public_key";
  private PublicKey dhEphemeralPublicKey;

  public static final String JSON_PROPERTY_DECRYPTORS = "decryptors";
  private List<EncryptedMessageDecryptor> decryptors = new ArrayList<>();

  public EncryptedMessageCurveDecryptorSet() { 
  }

  public EncryptedMessageCurveDecryptorSet dhEphemeralPublicKey(PublicKey dhEphemeralPublicKey) {
    this.dhEphemeralPublicKey = dhEphemeralPublicKey;
    return this;
  }

   /**
   * Get dhEphemeralPublicKey
   * @return dhEphemeralPublicKey
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_DH_EPHEMERAL_PUBLIC_KEY)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public PublicKey getDhEphemeralPublicKey() {
    return dhEphemeralPublicKey;
  }


  @JsonProperty(JSON_PROPERTY_DH_EPHEMERAL_PUBLIC_KEY)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setDhEphemeralPublicKey(PublicKey dhEphemeralPublicKey) {
    this.dhEphemeralPublicKey = dhEphemeralPublicKey;
  }


  public EncryptedMessageCurveDecryptorSet decryptors(List<EncryptedMessageDecryptor> decryptors) {
    this.decryptors = decryptors;
    return this;
  }

  public EncryptedMessageCurveDecryptorSet addDecryptorsItem(EncryptedMessageDecryptor decryptorsItem) {
    this.decryptors.add(decryptorsItem);
    return this;
  }

   /**
   * Get decryptors
   * @return decryptors
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_DECRYPTORS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public List<EncryptedMessageDecryptor> getDecryptors() {
    return decryptors;
  }


  @JsonProperty(JSON_PROPERTY_DECRYPTORS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setDecryptors(List<EncryptedMessageDecryptor> decryptors) {
    this.decryptors = decryptors;
  }


  /**
   * Return true if this EncryptedMessageCurveDecryptorSet object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    EncryptedMessageCurveDecryptorSet encryptedMessageCurveDecryptorSet = (EncryptedMessageCurveDecryptorSet) o;
    return Objects.equals(this.dhEphemeralPublicKey, encryptedMessageCurveDecryptorSet.dhEphemeralPublicKey) &&
        Objects.equals(this.decryptors, encryptedMessageCurveDecryptorSet.decryptors);
  }

  @Override
  public int hashCode() {
    return Objects.hash(dhEphemeralPublicKey, decryptors);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class EncryptedMessageCurveDecryptorSet {\n");
    sb.append("    dhEphemeralPublicKey: ").append(toIndentedString(dhEphemeralPublicKey)).append("\n");
    sb.append("    decryptors: ").append(toIndentedString(decryptors)).append("\n");
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

}

