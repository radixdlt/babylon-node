/*
 * Radix System API - Babylon (Anemone)
 * This API is exposed by the Babylon Radix node to give clients access to information about the node itself, its configuration, status and subsystems.  It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Heavy load may impact the node's function.  If you require queries against ledger state, you may also wish to consider using the [Core API or Gateway API instead](https://docs-babylon.radixdlt.com/main/apis/api-specification.html). 
 *
 * The version of the OpenAPI document: v1.1.0
 * 
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */


package com.radixdlt.api.system.generated.models;

import java.util.Objects;
import java.util.Arrays;
import java.util.Map;
import java.util.HashMap;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonTypeName;
import com.fasterxml.jackson.annotation.JsonValue;
import com.radixdlt.api.system.generated.models.SignalledReadinessThreshold;
import com.radixdlt.api.system.generated.models.SignalledReadinessThresholdState;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * SignalledReadinessPendingProtocolUpdateStateAllOfThresholdsState
 */
@JsonPropertyOrder({
  SignalledReadinessPendingProtocolUpdateStateAllOfThresholdsState.JSON_PROPERTY_THRESHOLD,
  SignalledReadinessPendingProtocolUpdateStateAllOfThresholdsState.JSON_PROPERTY_THRESHOLD_STATE
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class SignalledReadinessPendingProtocolUpdateStateAllOfThresholdsState {
  public static final String JSON_PROPERTY_THRESHOLD = "threshold";
  private SignalledReadinessThreshold threshold;

  public static final String JSON_PROPERTY_THRESHOLD_STATE = "threshold_state";
  private SignalledReadinessThresholdState thresholdState;


  public SignalledReadinessPendingProtocolUpdateStateAllOfThresholdsState threshold(SignalledReadinessThreshold threshold) {
    this.threshold = threshold;
    return this;
  }

   /**
   * Get threshold
   * @return threshold
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_THRESHOLD)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public SignalledReadinessThreshold getThreshold() {
    return threshold;
  }


  @JsonProperty(JSON_PROPERTY_THRESHOLD)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setThreshold(SignalledReadinessThreshold threshold) {
    this.threshold = threshold;
  }


  public SignalledReadinessPendingProtocolUpdateStateAllOfThresholdsState thresholdState(SignalledReadinessThresholdState thresholdState) {
    this.thresholdState = thresholdState;
    return this;
  }

   /**
   * Get thresholdState
   * @return thresholdState
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_THRESHOLD_STATE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public SignalledReadinessThresholdState getThresholdState() {
    return thresholdState;
  }


  @JsonProperty(JSON_PROPERTY_THRESHOLD_STATE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setThresholdState(SignalledReadinessThresholdState thresholdState) {
    this.thresholdState = thresholdState;
  }


  /**
   * Return true if this SignalledReadinessPendingProtocolUpdateState_allOf_thresholds_state object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    SignalledReadinessPendingProtocolUpdateStateAllOfThresholdsState signalledReadinessPendingProtocolUpdateStateAllOfThresholdsState = (SignalledReadinessPendingProtocolUpdateStateAllOfThresholdsState) o;
    return Objects.equals(this.threshold, signalledReadinessPendingProtocolUpdateStateAllOfThresholdsState.threshold) &&
        Objects.equals(this.thresholdState, signalledReadinessPendingProtocolUpdateStateAllOfThresholdsState.thresholdState);
  }

  @Override
  public int hashCode() {
    return Objects.hash(threshold, thresholdState);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class SignalledReadinessPendingProtocolUpdateStateAllOfThresholdsState {\n");
    sb.append("    threshold: ").append(toIndentedString(threshold)).append("\n");
    sb.append("    thresholdState: ").append(toIndentedString(thresholdState)).append("\n");
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

