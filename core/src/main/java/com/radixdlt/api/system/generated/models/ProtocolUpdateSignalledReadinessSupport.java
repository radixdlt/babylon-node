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
import com.fasterxml.jackson.annotation.JsonSubTypes;
import com.fasterxml.jackson.annotation.JsonTypeInfo;
import com.fasterxml.jackson.annotation.JsonTypeName;
import com.fasterxml.jackson.annotation.JsonValue;
import com.radixdlt.api.system.generated.models.ProtocolUpdateSignalledReadinessSupport;
import com.radixdlt.api.system.generated.models.ProtocolUpdateSignalledReadinessSupportAllOf;
import com.radixdlt.api.system.generated.models.ProtocolUpdateSupportType;
import com.radixdlt.api.system.generated.models.ProtocolUpdateSupportTypeDiscriminator;
import com.radixdlt.api.system.generated.models.SignalledReadinessThreshold;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import java.util.ArrayList;
import java.util.List;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


import com.radixdlt.api.common.JSON;
/**
 * ProtocolUpdateSignalledReadinessSupport
 */
@JsonPropertyOrder({
  ProtocolUpdateSignalledReadinessSupport.JSON_PROPERTY_THRESHOLDS
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
@JsonTypeInfo(use = JsonTypeInfo.Id.NAME, include = JsonTypeInfo.As.EXISTING_PROPERTY, property = "type", visible = true)
@JsonSubTypes({
  @JsonSubTypes.Type(value = ProtocolUpdateSignalledReadinessSupport.class, name = "SignalledReadiness"),
})

public class ProtocolUpdateSignalledReadinessSupport extends ProtocolUpdateSupportType {
  public static final String JSON_PROPERTY_THRESHOLDS = "thresholds";
  private List<SignalledReadinessThreshold> thresholds = new ArrayList<>();


  public ProtocolUpdateSignalledReadinessSupport thresholds(List<SignalledReadinessThreshold> thresholds) {
    this.thresholds = thresholds;
    return this;
  }

  public ProtocolUpdateSignalledReadinessSupport addThresholdsItem(SignalledReadinessThreshold thresholdsItem) {
    this.thresholds.add(thresholdsItem);
    return this;
  }

   /**
   * Get thresholds
   * @return thresholds
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_THRESHOLDS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public List<SignalledReadinessThreshold> getThresholds() {
    return thresholds;
  }


  @JsonProperty(JSON_PROPERTY_THRESHOLDS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setThresholds(List<SignalledReadinessThreshold> thresholds) {
    this.thresholds = thresholds;
  }


  /**
   * Return true if this ProtocolUpdateSignalledReadinessSupport object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    ProtocolUpdateSignalledReadinessSupport protocolUpdateSignalledReadinessSupport = (ProtocolUpdateSignalledReadinessSupport) o;
    return Objects.equals(this.thresholds, protocolUpdateSignalledReadinessSupport.thresholds) &&
        super.equals(o);
  }

  @Override
  public int hashCode() {
    return Objects.hash(thresholds, super.hashCode());
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class ProtocolUpdateSignalledReadinessSupport {\n");
    sb.append("    ").append(toIndentedString(super.toString())).append("\n");
    sb.append("    thresholds: ").append(toIndentedString(thresholds)).append("\n");
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
  mappings.put("SignalledReadiness", ProtocolUpdateSignalledReadinessSupport.class);
  mappings.put("ProtocolUpdateSignalledReadinessSupport", ProtocolUpdateSignalledReadinessSupport.class);
  JSON.registerDiscriminator(ProtocolUpdateSignalledReadinessSupport.class, "type", mappings);
}
}

