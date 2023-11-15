/*
 * Radix Core API - Babylon
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.0.4
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
import com.radixdlt.api.core.generated.models.ModuleId;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * BrowseObjectFieldRequest
 */
@JsonPropertyOrder({
  BrowseObjectFieldRequest.JSON_PROPERTY_NETWORK,
  BrowseObjectFieldRequest.JSON_PROPERTY_ENTITY_ADDRESS,
  BrowseObjectFieldRequest.JSON_PROPERTY_MODULE_ID,
  BrowseObjectFieldRequest.JSON_PROPERTY_FIELD_NAME,
  BrowseObjectFieldRequest.JSON_PROPERTY_FIELD_INDEX
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class BrowseObjectFieldRequest {
  public static final String JSON_PROPERTY_NETWORK = "network";
  private String network;

  public static final String JSON_PROPERTY_ENTITY_ADDRESS = "entity_address";
  private String entityAddress;

  public static final String JSON_PROPERTY_MODULE_ID = "module_id";
  private ModuleId moduleId;

  public static final String JSON_PROPERTY_FIELD_NAME = "field_name";
  private String fieldName;

  public static final String JSON_PROPERTY_FIELD_INDEX = "field_index";
  private Integer fieldIndex;

  public BrowseObjectFieldRequest() { 
  }

  public BrowseObjectFieldRequest network(String network) {
    this.network = network;
    return this;
  }

   /**
   * The logical name of the network
   * @return network
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(example = "{{network}}", required = true, value = "The logical name of the network")
  @JsonProperty(JSON_PROPERTY_NETWORK)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getNetwork() {
    return network;
  }


  @JsonProperty(JSON_PROPERTY_NETWORK)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setNetwork(String network) {
    this.network = network;
  }


  public BrowseObjectFieldRequest entityAddress(String entityAddress) {
    this.entityAddress = entityAddress;
    return this;
  }

   /**
   * Bech32m-encoded human readable version of the entity&#39;s address (ie the entity&#39;s node id)
   * @return entityAddress
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "Bech32m-encoded human readable version of the entity's address (ie the entity's node id)")
  @JsonProperty(JSON_PROPERTY_ENTITY_ADDRESS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getEntityAddress() {
    return entityAddress;
  }


  @JsonProperty(JSON_PROPERTY_ENTITY_ADDRESS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setEntityAddress(String entityAddress) {
    this.entityAddress = entityAddress;
  }


  public BrowseObjectFieldRequest moduleId(ModuleId moduleId) {
    this.moduleId = moduleId;
    return this;
  }

   /**
   * Get moduleId
   * @return moduleId
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "")
  @JsonProperty(JSON_PROPERTY_MODULE_ID)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public ModuleId getModuleId() {
    return moduleId;
  }


  @JsonProperty(JSON_PROPERTY_MODULE_ID)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setModuleId(ModuleId moduleId) {
    this.moduleId = moduleId;
  }


  public BrowseObjectFieldRequest fieldName(String fieldName) {
    this.fieldName = fieldName;
    return this;
  }

   /**
   * Name of the field to read. Either this or &#x60;field_index&#x60; is required.
   * @return fieldName
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "Name of the field to read. Either this or `field_index` is required.")
  @JsonProperty(JSON_PROPERTY_FIELD_NAME)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public String getFieldName() {
    return fieldName;
  }


  @JsonProperty(JSON_PROPERTY_FIELD_NAME)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setFieldName(String fieldName) {
    this.fieldName = fieldName;
  }


  public BrowseObjectFieldRequest fieldIndex(Integer fieldIndex) {
    this.fieldIndex = fieldIndex;
    return this;
  }

   /**
   * Index of the field to read. Either this or &#x60;field_name&#x60; is required.
   * minimum: 0
   * maximum: 255
   * @return fieldIndex
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "Index of the field to read. Either this or `field_name` is required.")
  @JsonProperty(JSON_PROPERTY_FIELD_INDEX)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public Integer getFieldIndex() {
    return fieldIndex;
  }


  @JsonProperty(JSON_PROPERTY_FIELD_INDEX)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setFieldIndex(Integer fieldIndex) {
    this.fieldIndex = fieldIndex;
  }


  /**
   * Return true if this BrowseObjectFieldRequest object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    BrowseObjectFieldRequest browseObjectFieldRequest = (BrowseObjectFieldRequest) o;
    return Objects.equals(this.network, browseObjectFieldRequest.network) &&
        Objects.equals(this.entityAddress, browseObjectFieldRequest.entityAddress) &&
        Objects.equals(this.moduleId, browseObjectFieldRequest.moduleId) &&
        Objects.equals(this.fieldName, browseObjectFieldRequest.fieldName) &&
        Objects.equals(this.fieldIndex, browseObjectFieldRequest.fieldIndex);
  }

  @Override
  public int hashCode() {
    return Objects.hash(network, entityAddress, moduleId, fieldName, fieldIndex);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class BrowseObjectFieldRequest {\n");
    sb.append("    network: ").append(toIndentedString(network)).append("\n");
    sb.append("    entityAddress: ").append(toIndentedString(entityAddress)).append("\n");
    sb.append("    moduleId: ").append(toIndentedString(moduleId)).append("\n");
    sb.append("    fieldName: ").append(toIndentedString(fieldName)).append("\n");
    sb.append("    fieldIndex: ").append(toIndentedString(fieldIndex)).append("\n");
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
