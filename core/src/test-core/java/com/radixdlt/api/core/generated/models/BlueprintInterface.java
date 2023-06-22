/*
 * Babylon Core API - RCnet V2
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  This version of the Core API belongs to the first release candidate of the Radix Babylon network (\"RCnet-V1\").  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` are guaranteed to be forward compatible to Babylon mainnet launch (and beyond). We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  We give no guarantees that other endpoints will not change before Babylon mainnet launch, although changes are expected to be minimal. 
 *
 * The version of the OpenAPI document: 0.4.0
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
import com.radixdlt.api.core.generated.models.FunctionSchema;
import com.radixdlt.api.core.generated.models.GenericType;
import com.radixdlt.api.core.generated.models.IndexedStateSchema;
import com.radixdlt.api.core.generated.models.TypePointer;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * BlueprintInterface
 */
@JsonPropertyOrder({
  BlueprintInterface.JSON_PROPERTY_OUTER_BLUEPRINT,
  BlueprintInterface.JSON_PROPERTY_GENERICS,
  BlueprintInterface.JSON_PROPERTY_FEATURES,
  BlueprintInterface.JSON_PROPERTY_STATE,
  BlueprintInterface.JSON_PROPERTY_FUNCTIONS,
  BlueprintInterface.JSON_PROPERTY_EVENTS
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class BlueprintInterface {
  public static final String JSON_PROPERTY_OUTER_BLUEPRINT = "outer_blueprint";
  private String outerBlueprint;

  public static final String JSON_PROPERTY_GENERICS = "generics";
  private List<GenericType> generics = new ArrayList<>();

  public static final String JSON_PROPERTY_FEATURES = "features";
  private List<String> features = new ArrayList<>();

  public static final String JSON_PROPERTY_STATE = "state";
  private IndexedStateSchema state;

  public static final String JSON_PROPERTY_FUNCTIONS = "functions";
  private Map<String, FunctionSchema> functions = new HashMap<>();

  public static final String JSON_PROPERTY_EVENTS = "events";
  private Map<String, TypePointer> events = new HashMap<>();

  public BlueprintInterface() { 
  }

  public BlueprintInterface outerBlueprint(String outerBlueprint) {
    this.outerBlueprint = outerBlueprint;
    return this;
  }

   /**
   * Get outerBlueprint
   * @return outerBlueprint
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "")
  @JsonProperty(JSON_PROPERTY_OUTER_BLUEPRINT)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public String getOuterBlueprint() {
    return outerBlueprint;
  }


  @JsonProperty(JSON_PROPERTY_OUTER_BLUEPRINT)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setOuterBlueprint(String outerBlueprint) {
    this.outerBlueprint = outerBlueprint;
  }


  public BlueprintInterface generics(List<GenericType> generics) {
    this.generics = generics;
    return this;
  }

  public BlueprintInterface addGenericsItem(GenericType genericsItem) {
    this.generics.add(genericsItem);
    return this;
  }

   /**
   * Get generics
   * @return generics
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_GENERICS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public List<GenericType> getGenerics() {
    return generics;
  }


  @JsonProperty(JSON_PROPERTY_GENERICS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setGenerics(List<GenericType> generics) {
    this.generics = generics;
  }


  public BlueprintInterface features(List<String> features) {
    this.features = features;
    return this;
  }

  public BlueprintInterface addFeaturesItem(String featuresItem) {
    this.features.add(featuresItem);
    return this;
  }

   /**
   * Get features
   * @return features
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_FEATURES)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public List<String> getFeatures() {
    return features;
  }


  @JsonProperty(JSON_PROPERTY_FEATURES)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setFeatures(List<String> features) {
    this.features = features;
  }


  public BlueprintInterface state(IndexedStateSchema state) {
    this.state = state;
    return this;
  }

   /**
   * Get state
   * @return state
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_STATE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public IndexedStateSchema getState() {
    return state;
  }


  @JsonProperty(JSON_PROPERTY_STATE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setState(IndexedStateSchema state) {
    this.state = state;
  }


  public BlueprintInterface functions(Map<String, FunctionSchema> functions) {
    this.functions = functions;
    return this;
  }

  public BlueprintInterface putFunctionsItem(String key, FunctionSchema functionsItem) {
    this.functions.put(key, functionsItem);
    return this;
  }

   /**
   * A map from the function name to the FunctionSchema
   * @return functions
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "A map from the function name to the FunctionSchema")
  @JsonProperty(JSON_PROPERTY_FUNCTIONS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Map<String, FunctionSchema> getFunctions() {
    return functions;
  }


  @JsonProperty(JSON_PROPERTY_FUNCTIONS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setFunctions(Map<String, FunctionSchema> functions) {
    this.functions = functions;
  }


  public BlueprintInterface events(Map<String, TypePointer> events) {
    this.events = events;
    return this;
  }

  public BlueprintInterface putEventsItem(String key, TypePointer eventsItem) {
    this.events.put(key, eventsItem);
    return this;
  }

   /**
   * A map from the event name to the local type index for the event payload under the blueprint schema.
   * @return events
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "A map from the event name to the local type index for the event payload under the blueprint schema.")
  @JsonProperty(JSON_PROPERTY_EVENTS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Map<String, TypePointer> getEvents() {
    return events;
  }


  @JsonProperty(JSON_PROPERTY_EVENTS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setEvents(Map<String, TypePointer> events) {
    this.events = events;
  }


  /**
   * Return true if this BlueprintInterface object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    BlueprintInterface blueprintInterface = (BlueprintInterface) o;
    return Objects.equals(this.outerBlueprint, blueprintInterface.outerBlueprint) &&
        Objects.equals(this.generics, blueprintInterface.generics) &&
        Objects.equals(this.features, blueprintInterface.features) &&
        Objects.equals(this.state, blueprintInterface.state) &&
        Objects.equals(this.functions, blueprintInterface.functions) &&
        Objects.equals(this.events, blueprintInterface.events);
  }

  @Override
  public int hashCode() {
    return Objects.hash(outerBlueprint, generics, features, state, functions, events);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class BlueprintInterface {\n");
    sb.append("    outerBlueprint: ").append(toIndentedString(outerBlueprint)).append("\n");
    sb.append("    generics: ").append(toIndentedString(generics)).append("\n");
    sb.append("    features: ").append(toIndentedString(features)).append("\n");
    sb.append("    state: ").append(toIndentedString(state)).append("\n");
    sb.append("    functions: ").append(toIndentedString(functions)).append("\n");
    sb.append("    events: ").append(toIndentedString(events)).append("\n");
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

