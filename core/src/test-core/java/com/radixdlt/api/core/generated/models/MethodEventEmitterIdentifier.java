/*
 * Radix Core API
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.3.0
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
import com.radixdlt.api.core.generated.models.EntityReference;
import com.radixdlt.api.core.generated.models.EventEmitterIdentifier;
import com.radixdlt.api.core.generated.models.EventEmitterIdentifierType;
import com.radixdlt.api.core.generated.models.FunctionEventEmitterIdentifier;
import com.radixdlt.api.core.generated.models.MethodEventEmitterIdentifier;
import com.radixdlt.api.core.generated.models.MethodEventEmitterIdentifierAllOf;
import com.radixdlt.api.core.generated.models.ModuleId;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


import com.radixdlt.api.core.generated.client.JSON;
/**
 * MethodEventEmitterIdentifier
 */
@JsonPropertyOrder({
  MethodEventEmitterIdentifier.JSON_PROPERTY_ENTITY,
  MethodEventEmitterIdentifier.JSON_PROPERTY_OBJECT_MODULE_ID
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
@JsonIgnoreProperties(
  value = "type", // ignore manually set type, it will be automatically generated by Jackson during serialization
  allowSetters = true // allows the type to be set during deserialization
)
@JsonTypeInfo(use = JsonTypeInfo.Id.NAME, include = JsonTypeInfo.As.PROPERTY, property = "type", visible = true)
@JsonSubTypes({
  @JsonSubTypes.Type(value = FunctionEventEmitterIdentifier.class, name = "Function"),
  @JsonSubTypes.Type(value = MethodEventEmitterIdentifier.class, name = "Method"),
})

public class MethodEventEmitterIdentifier extends EventEmitterIdentifier {
  public static final String JSON_PROPERTY_ENTITY = "entity";
  private EntityReference entity;

  public static final String JSON_PROPERTY_OBJECT_MODULE_ID = "object_module_id";
  private ModuleId objectModuleId;

  public MethodEventEmitterIdentifier() { 
  }

  public MethodEventEmitterIdentifier entity(EntityReference entity) {
    this.entity = entity;
    return this;
  }

   /**
   * Get entity
   * @return entity
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_ENTITY)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public EntityReference getEntity() {
    return entity;
  }


  @JsonProperty(JSON_PROPERTY_ENTITY)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setEntity(EntityReference entity) {
    this.entity = entity;
  }


  public MethodEventEmitterIdentifier objectModuleId(ModuleId objectModuleId) {
    this.objectModuleId = objectModuleId;
    return this;
  }

   /**
   * Get objectModuleId
   * @return objectModuleId
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_OBJECT_MODULE_ID)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public ModuleId getObjectModuleId() {
    return objectModuleId;
  }


  @JsonProperty(JSON_PROPERTY_OBJECT_MODULE_ID)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setObjectModuleId(ModuleId objectModuleId) {
    this.objectModuleId = objectModuleId;
  }


  /**
   * Return true if this MethodEventEmitterIdentifier object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    MethodEventEmitterIdentifier methodEventEmitterIdentifier = (MethodEventEmitterIdentifier) o;
    return Objects.equals(this.entity, methodEventEmitterIdentifier.entity) &&
        Objects.equals(this.objectModuleId, methodEventEmitterIdentifier.objectModuleId) &&
        super.equals(o);
  }

  @Override
  public int hashCode() {
    return Objects.hash(entity, objectModuleId, super.hashCode());
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class MethodEventEmitterIdentifier {\n");
    sb.append("    ").append(toIndentedString(super.toString())).append("\n");
    sb.append("    entity: ").append(toIndentedString(entity)).append("\n");
    sb.append("    objectModuleId: ").append(toIndentedString(objectModuleId)).append("\n");
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
  mappings.put("Function", FunctionEventEmitterIdentifier.class);
  mappings.put("Method", MethodEventEmitterIdentifier.class);
  mappings.put("MethodEventEmitterIdentifier", MethodEventEmitterIdentifier.class);
  JSON.registerDiscriminator(MethodEventEmitterIdentifier.class, "type", mappings);
}
}

