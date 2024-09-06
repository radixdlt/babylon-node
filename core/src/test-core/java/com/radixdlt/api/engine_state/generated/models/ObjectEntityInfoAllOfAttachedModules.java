/*
 * Engine State API (Beta)
 * **This API is currently in Beta**  This specification may experience breaking changes as part of Babylon Node releases. Such changes will be clearly mentioned in the [babylon-node release notes](https://github.com/radixdlt/babylon-node/releases). We advise against using this API for business-critical integrations before the `version` indicated above becomes stable, which is expected in Q4 of 2024.  This API provides a complete view of the current ledger state, operating at a relatively low level (i.e. returning Entities' data and type information in a generic way, without interpreting specifics of different native or custom components).  It mirrors how the Radix Engine views the ledger state in its \"System\" layer, and thus can be useful for Scrypto developers, who need to inspect how the Engine models and stores their application's state, or how an interface / authentication scheme of another component looks like. 
 *
 * The version of the OpenAPI document: v1.2.2
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
import com.radixdlt.api.engine_state.generated.models.AttachedModuleId;
import com.radixdlt.api.engine_state.generated.models.ObjectModuleStateInfo;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * ObjectEntityInfoAllOfAttachedModules
 */
@JsonPropertyOrder({
  ObjectEntityInfoAllOfAttachedModules.JSON_PROPERTY_ATTACHED_MODULE_ID,
  ObjectEntityInfoAllOfAttachedModules.JSON_PROPERTY_STATE
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class ObjectEntityInfoAllOfAttachedModules {
  public static final String JSON_PROPERTY_ATTACHED_MODULE_ID = "attached_module_id";
  private AttachedModuleId attachedModuleId;

  public static final String JSON_PROPERTY_STATE = "state";
  private ObjectModuleStateInfo state;

  public ObjectEntityInfoAllOfAttachedModules() { 
  }

  public ObjectEntityInfoAllOfAttachedModules attachedModuleId(AttachedModuleId attachedModuleId) {
    this.attachedModuleId = attachedModuleId;
    return this;
  }

   /**
   * Get attachedModuleId
   * @return attachedModuleId
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_ATTACHED_MODULE_ID)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public AttachedModuleId getAttachedModuleId() {
    return attachedModuleId;
  }


  @JsonProperty(JSON_PROPERTY_ATTACHED_MODULE_ID)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setAttachedModuleId(AttachedModuleId attachedModuleId) {
    this.attachedModuleId = attachedModuleId;
  }


  public ObjectEntityInfoAllOfAttachedModules state(ObjectModuleStateInfo state) {
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

  public ObjectModuleStateInfo getState() {
    return state;
  }


  @JsonProperty(JSON_PROPERTY_STATE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setState(ObjectModuleStateInfo state) {
    this.state = state;
  }


  /**
   * Return true if this ObjectEntityInfo_allOf_attached_modules object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    ObjectEntityInfoAllOfAttachedModules objectEntityInfoAllOfAttachedModules = (ObjectEntityInfoAllOfAttachedModules) o;
    return Objects.equals(this.attachedModuleId, objectEntityInfoAllOfAttachedModules.attachedModuleId) &&
        Objects.equals(this.state, objectEntityInfoAllOfAttachedModules.state);
  }

  @Override
  public int hashCode() {
    return Objects.hash(attachedModuleId, state);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class ObjectEntityInfoAllOfAttachedModules {\n");
    sb.append("    attachedModuleId: ").append(toIndentedString(attachedModuleId)).append("\n");
    sb.append("    state: ").append(toIndentedString(state)).append("\n");
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

