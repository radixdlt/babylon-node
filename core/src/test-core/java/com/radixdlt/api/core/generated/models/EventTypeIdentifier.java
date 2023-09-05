/*
 * Babylon Core API - RCnet v3.1
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  This version of the Core API belongs to the fourth release candidate of the Radix Babylon network (\"RCnet v3.1\").  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` are guaranteed to be forward compatible to Babylon mainnet launch (and beyond). We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code. 
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
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonTypeName;
import com.fasterxml.jackson.annotation.JsonValue;
import com.radixdlt.api.core.generated.models.EventEmitterIdentifier;
import com.radixdlt.api.core.generated.models.PackageTypeReference;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * Identifier of a specific event schema.
 */
@ApiModel(description = "Identifier of a specific event schema.")
@JsonPropertyOrder({
  EventTypeIdentifier.JSON_PROPERTY_EMITTER,
  EventTypeIdentifier.JSON_PROPERTY_TYPE_REFERENCE,
  EventTypeIdentifier.JSON_PROPERTY_NAME
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class EventTypeIdentifier {
  public static final String JSON_PROPERTY_EMITTER = "emitter";
  private EventEmitterIdentifier emitter;

  public static final String JSON_PROPERTY_TYPE_REFERENCE = "type_reference";
  private PackageTypeReference typeReference;

  public static final String JSON_PROPERTY_NAME = "name";
  private String name;

  public EventTypeIdentifier() { 
  }

  public EventTypeIdentifier emitter(EventEmitterIdentifier emitter) {
    this.emitter = emitter;
    return this;
  }

   /**
   * Get emitter
   * @return emitter
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_EMITTER)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public EventEmitterIdentifier getEmitter() {
    return emitter;
  }


  @JsonProperty(JSON_PROPERTY_EMITTER)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setEmitter(EventEmitterIdentifier emitter) {
    this.emitter = emitter;
  }


  public EventTypeIdentifier typeReference(PackageTypeReference typeReference) {
    this.typeReference = typeReference;
    return this;
  }

   /**
   * Get typeReference
   * @return typeReference
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_TYPE_REFERENCE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public PackageTypeReference getTypeReference() {
    return typeReference;
  }


  @JsonProperty(JSON_PROPERTY_TYPE_REFERENCE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setTypeReference(PackageTypeReference typeReference) {
    this.typeReference = typeReference;
  }


  public EventTypeIdentifier name(String name) {
    this.name = name;
    return this;
  }

   /**
   * Get name
   * @return name
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_NAME)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getName() {
    return name;
  }


  @JsonProperty(JSON_PROPERTY_NAME)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setName(String name) {
    this.name = name;
  }


  /**
   * Return true if this EventTypeIdentifier object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    EventTypeIdentifier eventTypeIdentifier = (EventTypeIdentifier) o;
    return Objects.equals(this.emitter, eventTypeIdentifier.emitter) &&
        Objects.equals(this.typeReference, eventTypeIdentifier.typeReference) &&
        Objects.equals(this.name, eventTypeIdentifier.name);
  }

  @Override
  public int hashCode() {
    return Objects.hash(emitter, typeReference, name);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class EventTypeIdentifier {\n");
    sb.append("    emitter: ").append(toIndentedString(emitter)).append("\n");
    sb.append("    typeReference: ").append(toIndentedString(typeReference)).append("\n");
    sb.append("    name: ").append(toIndentedString(name)).append("\n");
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

