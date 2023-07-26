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
import com.radixdlt.api.core.generated.models.EncryptedMessageCurveDecryptorSet;
import com.radixdlt.api.core.generated.models.EncryptedTransactionMessage;
import com.radixdlt.api.core.generated.models.EncryptedTransactionMessageAllOf;
import com.radixdlt.api.core.generated.models.PlaintextTransactionMessage;
import com.radixdlt.api.core.generated.models.TransactionMessage;
import com.radixdlt.api.core.generated.models.TransactionMessageType;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import java.util.ArrayList;
import java.util.List;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


import com.radixdlt.api.core.generated.client.JSON;
/**
 * EncryptedTransactionMessage
 */
@JsonPropertyOrder({
  EncryptedTransactionMessage.JSON_PROPERTY_ENCRYPTED_HEX,
  EncryptedTransactionMessage.JSON_PROPERTY_CURVE_DECRYPTOR_SETS
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
@JsonIgnoreProperties(
  value = "type", // ignore manually set type, it will be automatically generated by Jackson during serialization
  allowSetters = true // allows the type to be set during deserialization
)
@JsonTypeInfo(use = JsonTypeInfo.Id.NAME, include = JsonTypeInfo.As.PROPERTY, property = "type", visible = true)
@JsonSubTypes({
  @JsonSubTypes.Type(value = EncryptedTransactionMessage.class, name = "Encrypted"),
  @JsonSubTypes.Type(value = PlaintextTransactionMessage.class, name = "Plaintext"),
})

public class EncryptedTransactionMessage extends TransactionMessage {
  public static final String JSON_PROPERTY_ENCRYPTED_HEX = "encrypted_hex";
  private String encryptedHex;

  public static final String JSON_PROPERTY_CURVE_DECRYPTOR_SETS = "curve_decryptor_sets";
  private List<EncryptedMessageCurveDecryptorSet> curveDecryptorSets = new ArrayList<>();

  public EncryptedTransactionMessage() { 
  }

  public EncryptedTransactionMessage encryptedHex(String encryptedHex) {
    this.encryptedHex = encryptedHex;
    return this;
  }

   /**
   * The hex-encoded (128-bit) AES-GCM encrypted bytes of an SBOR-encoded &#x60;PlaintextTransactionMessage&#x60;. The bytes are serialized as the concatenation &#x60;Nonce/IV (12 bytes) || Cipher (variable length) || Tag/MAC (16 bytes)&#x60;: 
   * @return encryptedHex
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The hex-encoded (128-bit) AES-GCM encrypted bytes of an SBOR-encoded `PlaintextTransactionMessage`. The bytes are serialized as the concatenation `Nonce/IV (12 bytes) || Cipher (variable length) || Tag/MAC (16 bytes)`: ")
  @JsonProperty(JSON_PROPERTY_ENCRYPTED_HEX)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getEncryptedHex() {
    return encryptedHex;
  }


  @JsonProperty(JSON_PROPERTY_ENCRYPTED_HEX)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setEncryptedHex(String encryptedHex) {
    this.encryptedHex = encryptedHex;
  }


  public EncryptedTransactionMessage curveDecryptorSets(List<EncryptedMessageCurveDecryptorSet> curveDecryptorSets) {
    this.curveDecryptorSets = curveDecryptorSets;
    return this;
  }

  public EncryptedTransactionMessage addCurveDecryptorSetsItem(EncryptedMessageCurveDecryptorSet curveDecryptorSetsItem) {
    this.curveDecryptorSets.add(curveDecryptorSetsItem);
    return this;
  }

   /**
   * Get curveDecryptorSets
   * @return curveDecryptorSets
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_CURVE_DECRYPTOR_SETS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public List<EncryptedMessageCurveDecryptorSet> getCurveDecryptorSets() {
    return curveDecryptorSets;
  }


  @JsonProperty(JSON_PROPERTY_CURVE_DECRYPTOR_SETS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setCurveDecryptorSets(List<EncryptedMessageCurveDecryptorSet> curveDecryptorSets) {
    this.curveDecryptorSets = curveDecryptorSets;
  }


  /**
   * Return true if this EncryptedTransactionMessage object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    EncryptedTransactionMessage encryptedTransactionMessage = (EncryptedTransactionMessage) o;
    return Objects.equals(this.encryptedHex, encryptedTransactionMessage.encryptedHex) &&
        Objects.equals(this.curveDecryptorSets, encryptedTransactionMessage.curveDecryptorSets) &&
        super.equals(o);
  }

  @Override
  public int hashCode() {
    return Objects.hash(encryptedHex, curveDecryptorSets, super.hashCode());
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class EncryptedTransactionMessage {\n");
    sb.append("    ").append(toIndentedString(super.toString())).append("\n");
    sb.append("    encryptedHex: ").append(toIndentedString(encryptedHex)).append("\n");
    sb.append("    curveDecryptorSets: ").append(toIndentedString(curveDecryptorSets)).append("\n");
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
  mappings.put("Encrypted", EncryptedTransactionMessage.class);
  mappings.put("Plaintext", PlaintextTransactionMessage.class);
  mappings.put("EncryptedTransactionMessage", EncryptedTransactionMessage.class);
  JSON.registerDiscriminator(EncryptedTransactionMessage.class, "type", mappings);
}
}

