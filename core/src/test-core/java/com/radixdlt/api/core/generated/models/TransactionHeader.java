/*
 * Radix Core API
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.2.3
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
import com.radixdlt.api.core.generated.models.PublicKey;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * TransactionHeader
 */
@JsonPropertyOrder({
  TransactionHeader.JSON_PROPERTY_NETWORK_ID,
  TransactionHeader.JSON_PROPERTY_START_EPOCH_INCLUSIVE,
  TransactionHeader.JSON_PROPERTY_END_EPOCH_EXCLUSIVE,
  TransactionHeader.JSON_PROPERTY_NONCE,
  TransactionHeader.JSON_PROPERTY_NOTARY_PUBLIC_KEY,
  TransactionHeader.JSON_PROPERTY_NOTARY_IS_SIGNATORY,
  TransactionHeader.JSON_PROPERTY_TIP_PERCENTAGE
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class TransactionHeader {
  public static final String JSON_PROPERTY_NETWORK_ID = "network_id";
  private Integer networkId;

  public static final String JSON_PROPERTY_START_EPOCH_INCLUSIVE = "start_epoch_inclusive";
  private Long startEpochInclusive;

  public static final String JSON_PROPERTY_END_EPOCH_EXCLUSIVE = "end_epoch_exclusive";
  private Long endEpochExclusive;

  public static final String JSON_PROPERTY_NONCE = "nonce";
  private Long nonce;

  public static final String JSON_PROPERTY_NOTARY_PUBLIC_KEY = "notary_public_key";
  private PublicKey notaryPublicKey;

  public static final String JSON_PROPERTY_NOTARY_IS_SIGNATORY = "notary_is_signatory";
  private Boolean notaryIsSignatory;

  public static final String JSON_PROPERTY_TIP_PERCENTAGE = "tip_percentage";
  private Integer tipPercentage;

  public TransactionHeader() { 
  }

  public TransactionHeader networkId(Integer networkId) {
    this.networkId = networkId;
    return this;
  }

   /**
   * The logical id of the network
   * minimum: 0
   * maximum: 255
   * @return networkId
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The logical id of the network")
  @JsonProperty(JSON_PROPERTY_NETWORK_ID)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Integer getNetworkId() {
    return networkId;
  }


  @JsonProperty(JSON_PROPERTY_NETWORK_ID)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setNetworkId(Integer networkId) {
    this.networkId = networkId;
  }


  public TransactionHeader startEpochInclusive(Long startEpochInclusive) {
    this.startEpochInclusive = startEpochInclusive;
    return this;
  }

   /**
   * An integer between &#x60;0&#x60; and &#x60;10^10&#x60;, marking the epoch from which the transaction can be submitted. In the case of uncommitted transactions, a value of &#x60;10^10&#x60; indicates that the epoch was &gt;&#x3D; &#x60;10^10&#x60;. 
   * minimum: 0
   * maximum: 10000000000
   * @return startEpochInclusive
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "An integer between `0` and `10^10`, marking the epoch from which the transaction can be submitted. In the case of uncommitted transactions, a value of `10^10` indicates that the epoch was >= `10^10`. ")
  @JsonProperty(JSON_PROPERTY_START_EPOCH_INCLUSIVE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Long getStartEpochInclusive() {
    return startEpochInclusive;
  }


  @JsonProperty(JSON_PROPERTY_START_EPOCH_INCLUSIVE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setStartEpochInclusive(Long startEpochInclusive) {
    this.startEpochInclusive = startEpochInclusive;
  }


  public TransactionHeader endEpochExclusive(Long endEpochExclusive) {
    this.endEpochExclusive = endEpochExclusive;
    return this;
  }

   /**
   * An integer between &#x60;0&#x60; and &#x60;10^10&#x60;, marking the epoch from which the transaction will no longer be valid, and be rejected. In the case of uncommitted transactions, a value of &#x60;10^10&#x60; indicates that the epoch was &gt;&#x3D; &#x60;10^10&#x60;. 
   * minimum: 0
   * maximum: 10000000000
   * @return endEpochExclusive
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "An integer between `0` and `10^10`, marking the epoch from which the transaction will no longer be valid, and be rejected. In the case of uncommitted transactions, a value of `10^10` indicates that the epoch was >= `10^10`. ")
  @JsonProperty(JSON_PROPERTY_END_EPOCH_EXCLUSIVE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Long getEndEpochExclusive() {
    return endEpochExclusive;
  }


  @JsonProperty(JSON_PROPERTY_END_EPOCH_EXCLUSIVE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setEndEpochExclusive(Long endEpochExclusive) {
    this.endEpochExclusive = endEpochExclusive;
  }


  public TransactionHeader nonce(Long nonce) {
    this.nonce = nonce;
    return this;
  }

   /**
   * An integer between &#x60;0&#x60; and &#x60;2^32 - 1&#x60;, chosen to allow a unique intent to be created (to enable submitting an otherwise identical/duplicate intent).  As of Cuttlefish and V2 transaction models, this is now referred to in documentation as the &#x60;intent_discriminator&#x60;. 
   * minimum: 0
   * maximum: 4294967295
   * @return nonce
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "An integer between `0` and `2^32 - 1`, chosen to allow a unique intent to be created (to enable submitting an otherwise identical/duplicate intent).  As of Cuttlefish and V2 transaction models, this is now referred to in documentation as the `intent_discriminator`. ")
  @JsonProperty(JSON_PROPERTY_NONCE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Long getNonce() {
    return nonce;
  }


  @JsonProperty(JSON_PROPERTY_NONCE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setNonce(Long nonce) {
    this.nonce = nonce;
  }


  public TransactionHeader notaryPublicKey(PublicKey notaryPublicKey) {
    this.notaryPublicKey = notaryPublicKey;
    return this;
  }

   /**
   * Get notaryPublicKey
   * @return notaryPublicKey
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_NOTARY_PUBLIC_KEY)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public PublicKey getNotaryPublicKey() {
    return notaryPublicKey;
  }


  @JsonProperty(JSON_PROPERTY_NOTARY_PUBLIC_KEY)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setNotaryPublicKey(PublicKey notaryPublicKey) {
    this.notaryPublicKey = notaryPublicKey;
  }


  public TransactionHeader notaryIsSignatory(Boolean notaryIsSignatory) {
    this.notaryIsSignatory = notaryIsSignatory;
    return this;
  }

   /**
   * Specifies whether the notary public key should be included in the transaction signers list
   * @return notaryIsSignatory
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "Specifies whether the notary public key should be included in the transaction signers list")
  @JsonProperty(JSON_PROPERTY_NOTARY_IS_SIGNATORY)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Boolean getNotaryIsSignatory() {
    return notaryIsSignatory;
  }


  @JsonProperty(JSON_PROPERTY_NOTARY_IS_SIGNATORY)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setNotaryIsSignatory(Boolean notaryIsSignatory) {
    this.notaryIsSignatory = notaryIsSignatory;
  }


  public TransactionHeader tipPercentage(Integer tipPercentage) {
    this.tipPercentage = tipPercentage;
    return this;
  }

   /**
   * An integer between &#x60;0&#x60; and &#x60;65535&#x60;, giving the validator tip as a percentage amount. A value of &#x60;1&#x60; corresponds to 1% of the fee.
   * minimum: 0
   * maximum: 65535
   * @return tipPercentage
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "An integer between `0` and `65535`, giving the validator tip as a percentage amount. A value of `1` corresponds to 1% of the fee.")
  @JsonProperty(JSON_PROPERTY_TIP_PERCENTAGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Integer getTipPercentage() {
    return tipPercentage;
  }


  @JsonProperty(JSON_PROPERTY_TIP_PERCENTAGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setTipPercentage(Integer tipPercentage) {
    this.tipPercentage = tipPercentage;
  }


  /**
   * Return true if this TransactionHeader object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    TransactionHeader transactionHeader = (TransactionHeader) o;
    return Objects.equals(this.networkId, transactionHeader.networkId) &&
        Objects.equals(this.startEpochInclusive, transactionHeader.startEpochInclusive) &&
        Objects.equals(this.endEpochExclusive, transactionHeader.endEpochExclusive) &&
        Objects.equals(this.nonce, transactionHeader.nonce) &&
        Objects.equals(this.notaryPublicKey, transactionHeader.notaryPublicKey) &&
        Objects.equals(this.notaryIsSignatory, transactionHeader.notaryIsSignatory) &&
        Objects.equals(this.tipPercentage, transactionHeader.tipPercentage);
  }

  @Override
  public int hashCode() {
    return Objects.hash(networkId, startEpochInclusive, endEpochExclusive, nonce, notaryPublicKey, notaryIsSignatory, tipPercentage);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class TransactionHeader {\n");
    sb.append("    networkId: ").append(toIndentedString(networkId)).append("\n");
    sb.append("    startEpochInclusive: ").append(toIndentedString(startEpochInclusive)).append("\n");
    sb.append("    endEpochExclusive: ").append(toIndentedString(endEpochExclusive)).append("\n");
    sb.append("    nonce: ").append(toIndentedString(nonce)).append("\n");
    sb.append("    notaryPublicKey: ").append(toIndentedString(notaryPublicKey)).append("\n");
    sb.append("    notaryIsSignatory: ").append(toIndentedString(notaryIsSignatory)).append("\n");
    sb.append("    tipPercentage: ").append(toIndentedString(tipPercentage)).append("\n");
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

