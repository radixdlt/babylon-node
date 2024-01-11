/*
 * Radix System API - Babylon
 * This API is exposed by the Babylon Radix node to give clients access to information about the node itself, its configuration, status and subsystems.  It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Heavy load may impact the node's function.  If you require queries against ledger state, you may also wish to consider using the [Core API or Gateway API instead](https://docs-babylon.radixdlt.com/main/apis/api-specification.html). 
 *
 * The version of the OpenAPI document: v1.0.0
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
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import java.util.ArrayList;
import java.util.List;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * EnactWhenSupportedAndWithinBoundsConditionAllOf
 */
@JsonPropertyOrder({
  EnactWhenSupportedAndWithinBoundsConditionAllOf.JSON_PROPERTY_LOWER_BOUND_EPOCH,
  EnactWhenSupportedAndWithinBoundsConditionAllOf.JSON_PROPERTY_UPPER_BOUND_EPOCH,
  EnactWhenSupportedAndWithinBoundsConditionAllOf.JSON_PROPERTY_READINESS_THRESHOLDS
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class EnactWhenSupportedAndWithinBoundsConditionAllOf {
  public static final String JSON_PROPERTY_LOWER_BOUND_EPOCH = "lower_bound_epoch";
  private Long lowerBoundEpoch;

  public static final String JSON_PROPERTY_UPPER_BOUND_EPOCH = "upper_bound_epoch";
  private Long upperBoundEpoch;

  public static final String JSON_PROPERTY_READINESS_THRESHOLDS = "readiness_thresholds";
  private List<SignalledReadinessThreshold> readinessThresholds = null;


  public EnactWhenSupportedAndWithinBoundsConditionAllOf lowerBoundEpoch(Long lowerBoundEpoch) {
    this.lowerBoundEpoch = lowerBoundEpoch;
    return this;
  }

   /**
   * Get lowerBoundEpoch
   * @return lowerBoundEpoch
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_LOWER_BOUND_EPOCH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Long getLowerBoundEpoch() {
    return lowerBoundEpoch;
  }


  @JsonProperty(JSON_PROPERTY_LOWER_BOUND_EPOCH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setLowerBoundEpoch(Long lowerBoundEpoch) {
    this.lowerBoundEpoch = lowerBoundEpoch;
  }


  public EnactWhenSupportedAndWithinBoundsConditionAllOf upperBoundEpoch(Long upperBoundEpoch) {
    this.upperBoundEpoch = upperBoundEpoch;
    return this;
  }

   /**
   * Get upperBoundEpoch
   * @return upperBoundEpoch
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_UPPER_BOUND_EPOCH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Long getUpperBoundEpoch() {
    return upperBoundEpoch;
  }


  @JsonProperty(JSON_PROPERTY_UPPER_BOUND_EPOCH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setUpperBoundEpoch(Long upperBoundEpoch) {
    this.upperBoundEpoch = upperBoundEpoch;
  }


  public EnactWhenSupportedAndWithinBoundsConditionAllOf readinessThresholds(List<SignalledReadinessThreshold> readinessThresholds) {
    this.readinessThresholds = readinessThresholds;
    return this;
  }

  public EnactWhenSupportedAndWithinBoundsConditionAllOf addReadinessThresholdsItem(SignalledReadinessThreshold readinessThresholdsItem) {
    if (this.readinessThresholds == null) {
      this.readinessThresholds = new ArrayList<>();
    }
    this.readinessThresholds.add(readinessThresholdsItem);
    return this;
  }

   /**
   * Get readinessThresholds
   * @return readinessThresholds
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "")
  @JsonProperty(JSON_PROPERTY_READINESS_THRESHOLDS)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public List<SignalledReadinessThreshold> getReadinessThresholds() {
    return readinessThresholds;
  }


  @JsonProperty(JSON_PROPERTY_READINESS_THRESHOLDS)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setReadinessThresholds(List<SignalledReadinessThreshold> readinessThresholds) {
    this.readinessThresholds = readinessThresholds;
  }


  /**
   * Return true if this EnactWhenSupportedAndWithinBoundsCondition_allOf object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    EnactWhenSupportedAndWithinBoundsConditionAllOf enactWhenSupportedAndWithinBoundsConditionAllOf = (EnactWhenSupportedAndWithinBoundsConditionAllOf) o;
    return Objects.equals(this.lowerBoundEpoch, enactWhenSupportedAndWithinBoundsConditionAllOf.lowerBoundEpoch) &&
        Objects.equals(this.upperBoundEpoch, enactWhenSupportedAndWithinBoundsConditionAllOf.upperBoundEpoch) &&
        Objects.equals(this.readinessThresholds, enactWhenSupportedAndWithinBoundsConditionAllOf.readinessThresholds);
  }

  @Override
  public int hashCode() {
    return Objects.hash(lowerBoundEpoch, upperBoundEpoch, readinessThresholds);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class EnactWhenSupportedAndWithinBoundsConditionAllOf {\n");
    sb.append("    lowerBoundEpoch: ").append(toIndentedString(lowerBoundEpoch)).append("\n");
    sb.append("    upperBoundEpoch: ").append(toIndentedString(upperBoundEpoch)).append("\n");
    sb.append("    readinessThresholds: ").append(toIndentedString(readinessThresholds)).append("\n");
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

