/*
 * Babylon Core API
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 0.1.0
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
import com.radixdlt.api.core.generated.models.PublicKeyType;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * EcdsaSecp256k1Signature
 */
@JsonPropertyOrder({
  EcdsaSecp256k1Signature.JSON_PROPERTY_KEY_TYPE,
  EcdsaSecp256k1Signature.JSON_PROPERTY_SIGNATURE_HEX
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class EcdsaSecp256k1Signature {
  public static final String JSON_PROPERTY_KEY_TYPE = "key_type";
  private PublicKeyType keyType;

  public static final String JSON_PROPERTY_SIGNATURE_HEX = "signature_hex";
  private String signatureHex;

  public EcdsaSecp256k1Signature() { 
  }

  public EcdsaSecp256k1Signature keyType(PublicKeyType keyType) {
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

  public PublicKeyType getKeyType() {
    return keyType;
  }


  @JsonProperty(JSON_PROPERTY_KEY_TYPE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setKeyType(PublicKeyType keyType) {
    this.keyType = keyType;
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
    return Objects.equals(this.keyType, ecdsaSecp256k1Signature.keyType) &&
        Objects.equals(this.signatureHex, ecdsaSecp256k1Signature.signatureHex);
  }

  @Override
  public int hashCode() {
    return Objects.hash(keyType, signatureHex);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class EcdsaSecp256k1Signature {\n");
    sb.append("    keyType: ").append(toIndentedString(keyType)).append("\n");
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

}

