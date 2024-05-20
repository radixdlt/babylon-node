/*
 * Radix System API - Babylon (Anemone)
 * This API is exposed by the Babylon Radix node to give clients access to information about the node itself, its configuration, status and subsystems.  It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Heavy load may impact the node's function.  If you require queries against ledger state, you may also wish to consider using the [Core API or Gateway API instead](https://docs-babylon.radixdlt.com/main/apis/api-specification.html). 
 *
 * The version of the OpenAPI document: v1.2.1
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
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * SignalledReadinessThresholdState
 */
@JsonPropertyOrder({
  SignalledReadinessThresholdState.JSON_PROPERTY_CONSECUTIVE_STARTED_EPOCHS_OF_SUPPORT
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class SignalledReadinessThresholdState {
  public static final String JSON_PROPERTY_CONSECUTIVE_STARTED_EPOCHS_OF_SUPPORT = "consecutive_started_epochs_of_support";
  private Long consecutiveStartedEpochsOfSupport;


  public SignalledReadinessThresholdState consecutiveStartedEpochsOfSupport(Long consecutiveStartedEpochsOfSupport) {
    this.consecutiveStartedEpochsOfSupport = consecutiveStartedEpochsOfSupport;
    return this;
  }

   /**
   * Get consecutiveStartedEpochsOfSupport
   * @return consecutiveStartedEpochsOfSupport
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_CONSECUTIVE_STARTED_EPOCHS_OF_SUPPORT)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Long getConsecutiveStartedEpochsOfSupport() {
    return consecutiveStartedEpochsOfSupport;
  }


  @JsonProperty(JSON_PROPERTY_CONSECUTIVE_STARTED_EPOCHS_OF_SUPPORT)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setConsecutiveStartedEpochsOfSupport(Long consecutiveStartedEpochsOfSupport) {
    this.consecutiveStartedEpochsOfSupport = consecutiveStartedEpochsOfSupport;
  }


  /**
   * Return true if this SignalledReadinessThresholdState object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    SignalledReadinessThresholdState signalledReadinessThresholdState = (SignalledReadinessThresholdState) o;
    return Objects.equals(this.consecutiveStartedEpochsOfSupport, signalledReadinessThresholdState.consecutiveStartedEpochsOfSupport);
  }

  @Override
  public int hashCode() {
    return Objects.hash(consecutiveStartedEpochsOfSupport);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class SignalledReadinessThresholdState {\n");
    sb.append("    consecutiveStartedEpochsOfSupport: ").append(toIndentedString(consecutiveStartedEpochsOfSupport)).append("\n");
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

