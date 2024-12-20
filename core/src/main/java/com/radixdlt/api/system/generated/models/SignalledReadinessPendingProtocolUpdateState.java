/*
 * Radix System API
 * This API is exposed by the Babylon Radix node to give clients access to information about the node itself, its configuration, status and subsystems.  It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Heavy load may impact the node's function.  If you require queries against ledger state, you may also wish to consider using the [Core API or Gateway API instead](https://docs-babylon.radixdlt.com/main/apis/api-specification.html). 
 *
 * The version of the OpenAPI document: v1.3.0
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
import com.radixdlt.api.system.generated.models.EmptyPendingProtocolUpdateState;
import com.radixdlt.api.system.generated.models.PendingProtocolUpdateState;
import com.radixdlt.api.system.generated.models.PendingProtocolUpdateStateType;
import com.radixdlt.api.system.generated.models.SignalledReadinessPendingProtocolUpdateState;
import com.radixdlt.api.system.generated.models.SignalledReadinessPendingProtocolUpdateStateAllOf;
import com.radixdlt.api.system.generated.models.SignalledReadinessPendingProtocolUpdateStateAllOfThresholdsState;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import java.util.ArrayList;
import java.util.List;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


import com.radixdlt.api.common.JSON;
/**
 * SignalledReadinessPendingProtocolUpdateState
 */
@JsonPropertyOrder({
  SignalledReadinessPendingProtocolUpdateState.JSON_PROPERTY_THRESHOLDS_STATE
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
@JsonTypeInfo(use = JsonTypeInfo.Id.NAME, include = JsonTypeInfo.As.EXISTING_PROPERTY, property = "type", visible = true)
@JsonSubTypes({
  @JsonSubTypes.Type(value = EmptyPendingProtocolUpdateState.class, name = "Empty"),
  @JsonSubTypes.Type(value = SignalledReadinessPendingProtocolUpdateState.class, name = "ForSignalledReadinessSupportCondition"),
})

public class SignalledReadinessPendingProtocolUpdateState extends PendingProtocolUpdateState {
  public static final String JSON_PROPERTY_THRESHOLDS_STATE = "thresholds_state";
  private List<SignalledReadinessPendingProtocolUpdateStateAllOfThresholdsState> thresholdsState = new ArrayList<>();


  public SignalledReadinessPendingProtocolUpdateState thresholdsState(List<SignalledReadinessPendingProtocolUpdateStateAllOfThresholdsState> thresholdsState) {
    this.thresholdsState = thresholdsState;
    return this;
  }

  public SignalledReadinessPendingProtocolUpdateState addThresholdsStateItem(SignalledReadinessPendingProtocolUpdateStateAllOfThresholdsState thresholdsStateItem) {
    this.thresholdsState.add(thresholdsStateItem);
    return this;
  }

   /**
   * Get thresholdsState
   * @return thresholdsState
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_THRESHOLDS_STATE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public List<SignalledReadinessPendingProtocolUpdateStateAllOfThresholdsState> getThresholdsState() {
    return thresholdsState;
  }


  @JsonProperty(JSON_PROPERTY_THRESHOLDS_STATE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setThresholdsState(List<SignalledReadinessPendingProtocolUpdateStateAllOfThresholdsState> thresholdsState) {
    this.thresholdsState = thresholdsState;
  }


  /**
   * Return true if this SignalledReadinessPendingProtocolUpdateState object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    SignalledReadinessPendingProtocolUpdateState signalledReadinessPendingProtocolUpdateState = (SignalledReadinessPendingProtocolUpdateState) o;
    return Objects.equals(this.thresholdsState, signalledReadinessPendingProtocolUpdateState.thresholdsState) &&
        super.equals(o);
  }

  @Override
  public int hashCode() {
    return Objects.hash(thresholdsState, super.hashCode());
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class SignalledReadinessPendingProtocolUpdateState {\n");
    sb.append("    ").append(toIndentedString(super.toString())).append("\n");
    sb.append("    thresholdsState: ").append(toIndentedString(thresholdsState)).append("\n");
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
  mappings.put("Empty", EmptyPendingProtocolUpdateState.class);
  mappings.put("ForSignalledReadinessSupportCondition", SignalledReadinessPendingProtocolUpdateState.class);
  mappings.put("SignalledReadinessPendingProtocolUpdateState", SignalledReadinessPendingProtocolUpdateState.class);
  JSON.registerDiscriminator(SignalledReadinessPendingProtocolUpdateState.class, "type", mappings);
}
}

