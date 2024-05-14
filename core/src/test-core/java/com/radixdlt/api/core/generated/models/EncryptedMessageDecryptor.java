/*
 * Radix Core API - Babylon (Bottlenose)
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.2.0
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
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * EncryptedMessageDecryptor
 */
@JsonPropertyOrder({
  EncryptedMessageDecryptor.JSON_PROPERTY_PUBLIC_KEY_FINGERPRINT_HEX,
  EncryptedMessageDecryptor.JSON_PROPERTY_AES_WRAPPED_KEY_HEX
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class EncryptedMessageDecryptor {
  public static final String JSON_PROPERTY_PUBLIC_KEY_FINGERPRINT_HEX = "public_key_fingerprint_hex";
  private String publicKeyFingerprintHex;

  public static final String JSON_PROPERTY_AES_WRAPPED_KEY_HEX = "aes_wrapped_key_hex";
  private String aesWrappedKeyHex;

  public EncryptedMessageDecryptor() { 
  }

  public EncryptedMessageDecryptor publicKeyFingerprintHex(String publicKeyFingerprintHex) {
    this.publicKeyFingerprintHex = publicKeyFingerprintHex;
    return this;
  }

   /**
   * The last 8 bytes of the Blake2b-256 hash of the public key bytes, in their standard Radix byte-serialization.
   * @return publicKeyFingerprintHex
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The last 8 bytes of the Blake2b-256 hash of the public key bytes, in their standard Radix byte-serialization.")
  @JsonProperty(JSON_PROPERTY_PUBLIC_KEY_FINGERPRINT_HEX)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getPublicKeyFingerprintHex() {
    return publicKeyFingerprintHex;
  }


  @JsonProperty(JSON_PROPERTY_PUBLIC_KEY_FINGERPRINT_HEX)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setPublicKeyFingerprintHex(String publicKeyFingerprintHex) {
    this.publicKeyFingerprintHex = publicKeyFingerprintHex;
  }


  public EncryptedMessageDecryptor aesWrappedKeyHex(String aesWrappedKeyHex) {
    this.aesWrappedKeyHex = aesWrappedKeyHex;
    return this;
  }

   /**
   * The hex-encoded wrapped key bytes from applying RFC 3394 (256-bit) AES-KeyWrap to the 128-bit message ephemeral public key, with the secret KEK provided by static Diffie-Helman between the decryptor public key, and the &#x60;dh_ephemeral_public_key&#x60; for that curve type. The bytes are serialized (according to RFC 3394) as the concatenation &#x60;IV (first 8 bytes) || Cipher (wrapped 128-bit key, encoded as two 64-bit blocks)&#x60;. 
   * @return aesWrappedKeyHex
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The hex-encoded wrapped key bytes from applying RFC 3394 (256-bit) AES-KeyWrap to the 128-bit message ephemeral public key, with the secret KEK provided by static Diffie-Helman between the decryptor public key, and the `dh_ephemeral_public_key` for that curve type. The bytes are serialized (according to RFC 3394) as the concatenation `IV (first 8 bytes) || Cipher (wrapped 128-bit key, encoded as two 64-bit blocks)`. ")
  @JsonProperty(JSON_PROPERTY_AES_WRAPPED_KEY_HEX)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getAesWrappedKeyHex() {
    return aesWrappedKeyHex;
  }


  @JsonProperty(JSON_PROPERTY_AES_WRAPPED_KEY_HEX)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setAesWrappedKeyHex(String aesWrappedKeyHex) {
    this.aesWrappedKeyHex = aesWrappedKeyHex;
  }


  /**
   * Return true if this EncryptedMessageDecryptor object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    EncryptedMessageDecryptor encryptedMessageDecryptor = (EncryptedMessageDecryptor) o;
    return Objects.equals(this.publicKeyFingerprintHex, encryptedMessageDecryptor.publicKeyFingerprintHex) &&
        Objects.equals(this.aesWrappedKeyHex, encryptedMessageDecryptor.aesWrappedKeyHex);
  }

  @Override
  public int hashCode() {
    return Objects.hash(publicKeyFingerprintHex, aesWrappedKeyHex);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class EncryptedMessageDecryptor {\n");
    sb.append("    publicKeyFingerprintHex: ").append(toIndentedString(publicKeyFingerprintHex)).append("\n");
    sb.append("    aesWrappedKeyHex: ").append(toIndentedString(aesWrappedKeyHex)).append("\n");
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

