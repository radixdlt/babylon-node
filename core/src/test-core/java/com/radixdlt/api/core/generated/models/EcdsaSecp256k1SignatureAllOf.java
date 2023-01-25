/*
 * Babylon Core API
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node. It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Heavy load may impact the node's function.  If you require queries against historical ledger state, you may also wish to consider using the [Gateway API](https://betanet-gateway.redoc.ly/). 
 *
 * The version of the OpenAPI document: 0.2.0
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
 * EcdsaSecp256k1SignatureAllOf
 */
@JsonPropertyOrder({
  EcdsaSecp256k1SignatureAllOf.JSON_PROPERTY_SIGNATURE_HEX
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class EcdsaSecp256k1SignatureAllOf {
  public static final String JSON_PROPERTY_SIGNATURE_HEX = "signature_hex";
  private String signatureHex;

  public EcdsaSecp256k1SignatureAllOf() { 
  }

  public EcdsaSecp256k1SignatureAllOf signatureHex(String signatureHex) {
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
   * Return true if this EcdsaSecp256k1Signature_allOf object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    EcdsaSecp256k1SignatureAllOf ecdsaSecp256k1SignatureAllOf = (EcdsaSecp256k1SignatureAllOf) o;
    return Objects.equals(this.signatureHex, ecdsaSecp256k1SignatureAllOf.signatureHex);
  }

  @Override
  public int hashCode() {
    return Objects.hash(signatureHex);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class EcdsaSecp256k1SignatureAllOf {\n");
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

