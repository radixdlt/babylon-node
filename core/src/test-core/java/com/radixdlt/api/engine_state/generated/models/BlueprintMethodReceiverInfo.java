/*
 * Engine State API
 * This API provides a complete view of the current ledger state, operating at a relatively low level (i.e. returning Entities' data and type information in a generic way, without interpreting specifics of different native or custom components).  It mirrors how the Radix Engine views the ledger state in its \"System\" layer, and thus can be useful for Scrypto developers, who need to inspect how the Engine models and stores their application's state, or how an interface / authentication scheme of another component looks like. 
 *
 * The version of the OpenAPI document: v0.0.1
 * 
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */


package com.radixdlt.api.engine_state.generated.models;

import java.util.Objects;
import java.util.Arrays;
import java.util.Map;
import java.util.HashMap;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonTypeName;
import com.fasterxml.jackson.annotation.JsonValue;
import com.radixdlt.api.engine_state.generated.models.MethodReceiverReferenceType;
import com.radixdlt.api.engine_state.generated.models.MethodReceiverType;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import java.util.ArrayList;
import java.util.List;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * BlueprintMethodReceiverInfo
 */
@JsonPropertyOrder({
  BlueprintMethodReceiverInfo.JSON_PROPERTY_RECEIVER_TYPE,
  BlueprintMethodReceiverInfo.JSON_PROPERTY_REFERENCE_TYPES
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class BlueprintMethodReceiverInfo {
  public static final String JSON_PROPERTY_RECEIVER_TYPE = "receiver_type";
  private MethodReceiverType receiverType;

  public static final String JSON_PROPERTY_REFERENCE_TYPES = "reference_types";
  private List<MethodReceiverReferenceType> referenceTypes = new ArrayList<>();

  public BlueprintMethodReceiverInfo() { 
  }

  public BlueprintMethodReceiverInfo receiverType(MethodReceiverType receiverType) {
    this.receiverType = receiverType;
    return this;
  }

   /**
   * Get receiverType
   * @return receiverType
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_RECEIVER_TYPE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public MethodReceiverType getReceiverType() {
    return receiverType;
  }


  @JsonProperty(JSON_PROPERTY_RECEIVER_TYPE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setReceiverType(MethodReceiverType receiverType) {
    this.receiverType = receiverType;
  }


  public BlueprintMethodReceiverInfo referenceTypes(List<MethodReceiverReferenceType> referenceTypes) {
    this.referenceTypes = referenceTypes;
    return this;
  }

  public BlueprintMethodReceiverInfo addReferenceTypesItem(MethodReceiverReferenceType referenceTypesItem) {
    this.referenceTypes.add(referenceTypesItem);
    return this;
  }

   /**
   * Get referenceTypes
   * @return referenceTypes
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_REFERENCE_TYPES)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public List<MethodReceiverReferenceType> getReferenceTypes() {
    return referenceTypes;
  }


  @JsonProperty(JSON_PROPERTY_REFERENCE_TYPES)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setReferenceTypes(List<MethodReceiverReferenceType> referenceTypes) {
    this.referenceTypes = referenceTypes;
  }


  /**
   * Return true if this BlueprintMethodReceiverInfo object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    BlueprintMethodReceiverInfo blueprintMethodReceiverInfo = (BlueprintMethodReceiverInfo) o;
    return Objects.equals(this.receiverType, blueprintMethodReceiverInfo.receiverType) &&
        Objects.equals(this.referenceTypes, blueprintMethodReceiverInfo.referenceTypes);
  }

  @Override
  public int hashCode() {
    return Objects.hash(receiverType, referenceTypes);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class BlueprintMethodReceiverInfo {\n");
    sb.append("    receiverType: ").append(toIndentedString(receiverType)).append("\n");
    sb.append("    referenceTypes: ").append(toIndentedString(referenceTypes)).append("\n");
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
