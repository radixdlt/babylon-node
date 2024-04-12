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
import com.radixdlt.api.core.generated.models.InstantMs;
import com.radixdlt.api.core.generated.models.LtsTransactionSubmitErrorDetails;
import com.radixdlt.api.core.generated.models.LtsTransactionSubmitErrorDetailsType;
import com.radixdlt.api.core.generated.models.LtsTransactionSubmitIntentAlreadyCommitted;
import com.radixdlt.api.core.generated.models.LtsTransactionSubmitPriorityThresholdNotMetErrorDetails;
import com.radixdlt.api.core.generated.models.LtsTransactionSubmitRejectedErrorDetails;
import com.radixdlt.api.core.generated.models.LtsTransactionSubmitRejectedErrorDetailsAllOf;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


import com.radixdlt.api.core.generated.client.JSON;
/**
 * LtsTransactionSubmitRejectedErrorDetails
 */
@JsonPropertyOrder({
  LtsTransactionSubmitRejectedErrorDetails.JSON_PROPERTY_ERROR_MESSAGE,
  LtsTransactionSubmitRejectedErrorDetails.JSON_PROPERTY_IS_FRESH,
  LtsTransactionSubmitRejectedErrorDetails.JSON_PROPERTY_IS_PAYLOAD_REJECTION_PERMANENT,
  LtsTransactionSubmitRejectedErrorDetails.JSON_PROPERTY_IS_INTENT_REJECTION_PERMANENT,
  LtsTransactionSubmitRejectedErrorDetails.JSON_PROPERTY_RETRY_FROM_TIMESTAMP,
  LtsTransactionSubmitRejectedErrorDetails.JSON_PROPERTY_RETRY_FROM_EPOCH,
  LtsTransactionSubmitRejectedErrorDetails.JSON_PROPERTY_INVALID_FROM_EPOCH
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
@JsonIgnoreProperties(
  value = "type", // ignore manually set type, it will be automatically generated by Jackson during serialization
  allowSetters = true // allows the type to be set during deserialization
)
@JsonTypeInfo(use = JsonTypeInfo.Id.NAME, include = JsonTypeInfo.As.PROPERTY, property = "type", visible = true)
@JsonSubTypes({
  @JsonSubTypes.Type(value = LtsTransactionSubmitIntentAlreadyCommitted.class, name = "IntentAlreadyCommitted"),
  @JsonSubTypes.Type(value = LtsTransactionSubmitPriorityThresholdNotMetErrorDetails.class, name = "PriorityThresholdNotMet"),
  @JsonSubTypes.Type(value = LtsTransactionSubmitRejectedErrorDetails.class, name = "Rejected"),
})

public class LtsTransactionSubmitRejectedErrorDetails extends LtsTransactionSubmitErrorDetails {
  public static final String JSON_PROPERTY_ERROR_MESSAGE = "error_message";
  private String errorMessage;

  public static final String JSON_PROPERTY_IS_FRESH = "is_fresh";
  private Boolean isFresh;

  public static final String JSON_PROPERTY_IS_PAYLOAD_REJECTION_PERMANENT = "is_payload_rejection_permanent";
  private Boolean isPayloadRejectionPermanent;

  public static final String JSON_PROPERTY_IS_INTENT_REJECTION_PERMANENT = "is_intent_rejection_permanent";
  private Boolean isIntentRejectionPermanent;

  public static final String JSON_PROPERTY_RETRY_FROM_TIMESTAMP = "retry_from_timestamp";
  private InstantMs retryFromTimestamp;

  public static final String JSON_PROPERTY_RETRY_FROM_EPOCH = "retry_from_epoch";
  private Long retryFromEpoch;

  public static final String JSON_PROPERTY_INVALID_FROM_EPOCH = "invalid_from_epoch";
  private Long invalidFromEpoch;

  public LtsTransactionSubmitRejectedErrorDetails() { 
  }

  public LtsTransactionSubmitRejectedErrorDetails errorMessage(String errorMessage) {
    this.errorMessage = errorMessage;
    return this;
  }

   /**
   * An explanation of the error
   * @return errorMessage
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "An explanation of the error")
  @JsonProperty(JSON_PROPERTY_ERROR_MESSAGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getErrorMessage() {
    return errorMessage;
  }


  @JsonProperty(JSON_PROPERTY_ERROR_MESSAGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setErrorMessage(String errorMessage) {
    this.errorMessage = errorMessage;
  }


  public LtsTransactionSubmitRejectedErrorDetails isFresh(Boolean isFresh) {
    this.isFresh = isFresh;
    return this;
  }

   /**
   * Whether (true) this rejected status has just been calculated fresh, or (false) the status is from the pending transaction result cache. 
   * @return isFresh
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "Whether (true) this rejected status has just been calculated fresh, or (false) the status is from the pending transaction result cache. ")
  @JsonProperty(JSON_PROPERTY_IS_FRESH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Boolean getIsFresh() {
    return isFresh;
  }


  @JsonProperty(JSON_PROPERTY_IS_FRESH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setIsFresh(Boolean isFresh) {
    this.isFresh = isFresh;
  }


  public LtsTransactionSubmitRejectedErrorDetails isPayloadRejectionPermanent(Boolean isPayloadRejectionPermanent) {
    this.isPayloadRejectionPermanent = isPayloadRejectionPermanent;
    return this;
  }

   /**
   * Whether the rejection of this payload is known to be permanent. 
   * @return isPayloadRejectionPermanent
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "Whether the rejection of this payload is known to be permanent. ")
  @JsonProperty(JSON_PROPERTY_IS_PAYLOAD_REJECTION_PERMANENT)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Boolean getIsPayloadRejectionPermanent() {
    return isPayloadRejectionPermanent;
  }


  @JsonProperty(JSON_PROPERTY_IS_PAYLOAD_REJECTION_PERMANENT)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setIsPayloadRejectionPermanent(Boolean isPayloadRejectionPermanent) {
    this.isPayloadRejectionPermanent = isPayloadRejectionPermanent;
  }


  public LtsTransactionSubmitRejectedErrorDetails isIntentRejectionPermanent(Boolean isIntentRejectionPermanent) {
    this.isIntentRejectionPermanent = isIntentRejectionPermanent;
    return this;
  }

   /**
   * Whether the rejection of this intent is known to be permanent - this is a stronger statement than the payload rejection being permanent, as it implies any payloads containing the intent will also be permanently rejected. 
   * @return isIntentRejectionPermanent
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "Whether the rejection of this intent is known to be permanent - this is a stronger statement than the payload rejection being permanent, as it implies any payloads containing the intent will also be permanently rejected. ")
  @JsonProperty(JSON_PROPERTY_IS_INTENT_REJECTION_PERMANENT)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Boolean getIsIntentRejectionPermanent() {
    return isIntentRejectionPermanent;
  }


  @JsonProperty(JSON_PROPERTY_IS_INTENT_REJECTION_PERMANENT)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setIsIntentRejectionPermanent(Boolean isIntentRejectionPermanent) {
    this.isIntentRejectionPermanent = isIntentRejectionPermanent;
  }


  public LtsTransactionSubmitRejectedErrorDetails retryFromTimestamp(InstantMs retryFromTimestamp) {
    this.retryFromTimestamp = retryFromTimestamp;
    return this;
  }

   /**
   * Get retryFromTimestamp
   * @return retryFromTimestamp
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "")
  @JsonProperty(JSON_PROPERTY_RETRY_FROM_TIMESTAMP)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public InstantMs getRetryFromTimestamp() {
    return retryFromTimestamp;
  }


  @JsonProperty(JSON_PROPERTY_RETRY_FROM_TIMESTAMP)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setRetryFromTimestamp(InstantMs retryFromTimestamp) {
    this.retryFromTimestamp = retryFromTimestamp;
  }


  public LtsTransactionSubmitRejectedErrorDetails retryFromEpoch(Long retryFromEpoch) {
    this.retryFromEpoch = retryFromEpoch;
    return this;
  }

   /**
   * An integer between &#x60;0&#x60; and &#x60;10^10&#x60;, marking the epoch after which the node will consider recalculating the validity of the transaction. Only present if the rejection is temporary due to a header specifying a \&quot;from epoch\&quot; in the future. 
   * minimum: 0
   * maximum: 10000000000
   * @return retryFromEpoch
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "An integer between `0` and `10^10`, marking the epoch after which the node will consider recalculating the validity of the transaction. Only present if the rejection is temporary due to a header specifying a \"from epoch\" in the future. ")
  @JsonProperty(JSON_PROPERTY_RETRY_FROM_EPOCH)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public Long getRetryFromEpoch() {
    return retryFromEpoch;
  }


  @JsonProperty(JSON_PROPERTY_RETRY_FROM_EPOCH)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setRetryFromEpoch(Long retryFromEpoch) {
    this.retryFromEpoch = retryFromEpoch;
  }


  public LtsTransactionSubmitRejectedErrorDetails invalidFromEpoch(Long invalidFromEpoch) {
    this.invalidFromEpoch = invalidFromEpoch;
    return this;
  }

   /**
   * An integer between &#x60;0&#x60; and &#x60;10^10&#x60;, marking the epoch from which the transaction will no longer be valid, and be permanently rejected. Only present if the rejection isn&#39;t permanent. 
   * minimum: 0
   * maximum: 10000000000
   * @return invalidFromEpoch
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "An integer between `0` and `10^10`, marking the epoch from which the transaction will no longer be valid, and be permanently rejected. Only present if the rejection isn't permanent. ")
  @JsonProperty(JSON_PROPERTY_INVALID_FROM_EPOCH)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public Long getInvalidFromEpoch() {
    return invalidFromEpoch;
  }


  @JsonProperty(JSON_PROPERTY_INVALID_FROM_EPOCH)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setInvalidFromEpoch(Long invalidFromEpoch) {
    this.invalidFromEpoch = invalidFromEpoch;
  }


  /**
   * Return true if this LtsTransactionSubmitRejectedErrorDetails object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    LtsTransactionSubmitRejectedErrorDetails ltsTransactionSubmitRejectedErrorDetails = (LtsTransactionSubmitRejectedErrorDetails) o;
    return Objects.equals(this.errorMessage, ltsTransactionSubmitRejectedErrorDetails.errorMessage) &&
        Objects.equals(this.isFresh, ltsTransactionSubmitRejectedErrorDetails.isFresh) &&
        Objects.equals(this.isPayloadRejectionPermanent, ltsTransactionSubmitRejectedErrorDetails.isPayloadRejectionPermanent) &&
        Objects.equals(this.isIntentRejectionPermanent, ltsTransactionSubmitRejectedErrorDetails.isIntentRejectionPermanent) &&
        Objects.equals(this.retryFromTimestamp, ltsTransactionSubmitRejectedErrorDetails.retryFromTimestamp) &&
        Objects.equals(this.retryFromEpoch, ltsTransactionSubmitRejectedErrorDetails.retryFromEpoch) &&
        Objects.equals(this.invalidFromEpoch, ltsTransactionSubmitRejectedErrorDetails.invalidFromEpoch) &&
        super.equals(o);
  }

  @Override
  public int hashCode() {
    return Objects.hash(errorMessage, isFresh, isPayloadRejectionPermanent, isIntentRejectionPermanent, retryFromTimestamp, retryFromEpoch, invalidFromEpoch, super.hashCode());
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class LtsTransactionSubmitRejectedErrorDetails {\n");
    sb.append("    ").append(toIndentedString(super.toString())).append("\n");
    sb.append("    errorMessage: ").append(toIndentedString(errorMessage)).append("\n");
    sb.append("    isFresh: ").append(toIndentedString(isFresh)).append("\n");
    sb.append("    isPayloadRejectionPermanent: ").append(toIndentedString(isPayloadRejectionPermanent)).append("\n");
    sb.append("    isIntentRejectionPermanent: ").append(toIndentedString(isIntentRejectionPermanent)).append("\n");
    sb.append("    retryFromTimestamp: ").append(toIndentedString(retryFromTimestamp)).append("\n");
    sb.append("    retryFromEpoch: ").append(toIndentedString(retryFromEpoch)).append("\n");
    sb.append("    invalidFromEpoch: ").append(toIndentedString(invalidFromEpoch)).append("\n");
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
  mappings.put("IntentAlreadyCommitted", LtsTransactionSubmitIntentAlreadyCommitted.class);
  mappings.put("PriorityThresholdNotMet", LtsTransactionSubmitPriorityThresholdNotMetErrorDetails.class);
  mappings.put("Rejected", LtsTransactionSubmitRejectedErrorDetails.class);
  mappings.put("LtsTransactionSubmitRejectedErrorDetails", LtsTransactionSubmitRejectedErrorDetails.class);
  JSON.registerDiscriminator(LtsTransactionSubmitRejectedErrorDetails.class, "type", mappings);
}
}

