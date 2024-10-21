/*
 * Radix Core API
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.2.3
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
import com.radixdlt.api.core.generated.models.ConsensusLedgerProofOrigin;
import com.radixdlt.api.core.generated.models.GenesisLedgerProofOrigin;
import com.radixdlt.api.core.generated.models.LedgerProofOrigin;
import com.radixdlt.api.core.generated.models.LedgerProofOriginType;
import com.radixdlt.api.core.generated.models.ProtocolUpdateLedgerProofOrigin;
import com.radixdlt.api.core.generated.models.ProtocolUpdateLedgerProofOriginAllOf;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


import com.radixdlt.api.core.generated.client.JSON;
/**
 * ProtocolUpdateLedgerProofOrigin
 */
@JsonPropertyOrder({
  ProtocolUpdateLedgerProofOrigin.JSON_PROPERTY_PROTOCOL_VERSION_NAME,
  ProtocolUpdateLedgerProofOrigin.JSON_PROPERTY_CONFIG_HASH,
  ProtocolUpdateLedgerProofOrigin.JSON_PROPERTY_BATCH_GROUP_IDX,
  ProtocolUpdateLedgerProofOrigin.JSON_PROPERTY_BATCH_GROUP_NAME,
  ProtocolUpdateLedgerProofOrigin.JSON_PROPERTY_BATCH_IDX,
  ProtocolUpdateLedgerProofOrigin.JSON_PROPERTY_BATCH_NAME,
  ProtocolUpdateLedgerProofOrigin.JSON_PROPERTY_IS_END_OF_UPDATE
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
@JsonIgnoreProperties(
  value = "type", // ignore manually set type, it will be automatically generated by Jackson during serialization
  allowSetters = true // allows the type to be set during deserialization
)
@JsonTypeInfo(use = JsonTypeInfo.Id.NAME, include = JsonTypeInfo.As.PROPERTY, property = "type", visible = true)
@JsonSubTypes({
  @JsonSubTypes.Type(value = ConsensusLedgerProofOrigin.class, name = "Consensus"),
  @JsonSubTypes.Type(value = GenesisLedgerProofOrigin.class, name = "Genesis"),
  @JsonSubTypes.Type(value = ProtocolUpdateLedgerProofOrigin.class, name = "ProtocolUpdate"),
})

public class ProtocolUpdateLedgerProofOrigin extends LedgerProofOrigin {
  public static final String JSON_PROPERTY_PROTOCOL_VERSION_NAME = "protocol_version_name";
  private String protocolVersionName;

  public static final String JSON_PROPERTY_CONFIG_HASH = "config_hash";
  private String configHash;

  public static final String JSON_PROPERTY_BATCH_GROUP_IDX = "batch_group_idx";
  private Long batchGroupIdx;

  public static final String JSON_PROPERTY_BATCH_GROUP_NAME = "batch_group_name";
  private String batchGroupName;

  public static final String JSON_PROPERTY_BATCH_IDX = "batch_idx";
  private Long batchIdx;

  public static final String JSON_PROPERTY_BATCH_NAME = "batch_name";
  private String batchName;

  public static final String JSON_PROPERTY_IS_END_OF_UPDATE = "is_end_of_update";
  private Boolean isEndOfUpdate;

  public ProtocolUpdateLedgerProofOrigin() { 
  }

  public ProtocolUpdateLedgerProofOrigin protocolVersionName(String protocolVersionName) {
    this.protocolVersionName = protocolVersionName;
    return this;
  }

   /**
   * Get protocolVersionName
   * @return protocolVersionName
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_PROTOCOL_VERSION_NAME)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getProtocolVersionName() {
    return protocolVersionName;
  }


  @JsonProperty(JSON_PROPERTY_PROTOCOL_VERSION_NAME)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setProtocolVersionName(String protocolVersionName) {
    this.protocolVersionName = protocolVersionName;
  }


  public ProtocolUpdateLedgerProofOrigin configHash(String configHash) {
    this.configHash = configHash;
    return this;
  }

   /**
   * Get configHash
   * @return configHash
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_CONFIG_HASH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getConfigHash() {
    return configHash;
  }


  @JsonProperty(JSON_PROPERTY_CONFIG_HASH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setConfigHash(String configHash) {
    this.configHash = configHash;
  }


  public ProtocolUpdateLedgerProofOrigin batchGroupIdx(Long batchGroupIdx) {
    this.batchGroupIdx = batchGroupIdx;
    return this;
  }

   /**
   * Get batchGroupIdx
   * @return batchGroupIdx
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_BATCH_GROUP_IDX)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Long getBatchGroupIdx() {
    return batchGroupIdx;
  }


  @JsonProperty(JSON_PROPERTY_BATCH_GROUP_IDX)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setBatchGroupIdx(Long batchGroupIdx) {
    this.batchGroupIdx = batchGroupIdx;
  }


  public ProtocolUpdateLedgerProofOrigin batchGroupName(String batchGroupName) {
    this.batchGroupName = batchGroupName;
    return this;
  }

   /**
   * Get batchGroupName
   * @return batchGroupName
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_BATCH_GROUP_NAME)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getBatchGroupName() {
    return batchGroupName;
  }


  @JsonProperty(JSON_PROPERTY_BATCH_GROUP_NAME)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setBatchGroupName(String batchGroupName) {
    this.batchGroupName = batchGroupName;
  }


  public ProtocolUpdateLedgerProofOrigin batchIdx(Long batchIdx) {
    this.batchIdx = batchIdx;
    return this;
  }

   /**
   * Get batchIdx
   * @return batchIdx
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_BATCH_IDX)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Long getBatchIdx() {
    return batchIdx;
  }


  @JsonProperty(JSON_PROPERTY_BATCH_IDX)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setBatchIdx(Long batchIdx) {
    this.batchIdx = batchIdx;
  }


  public ProtocolUpdateLedgerProofOrigin batchName(String batchName) {
    this.batchName = batchName;
    return this;
  }

   /**
   * Get batchName
   * @return batchName
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_BATCH_NAME)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getBatchName() {
    return batchName;
  }


  @JsonProperty(JSON_PROPERTY_BATCH_NAME)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setBatchName(String batchName) {
    this.batchName = batchName;
  }


  public ProtocolUpdateLedgerProofOrigin isEndOfUpdate(Boolean isEndOfUpdate) {
    this.isEndOfUpdate = isEndOfUpdate;
    return this;
  }

   /**
   * Get isEndOfUpdate
   * @return isEndOfUpdate
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_IS_END_OF_UPDATE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Boolean getIsEndOfUpdate() {
    return isEndOfUpdate;
  }


  @JsonProperty(JSON_PROPERTY_IS_END_OF_UPDATE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setIsEndOfUpdate(Boolean isEndOfUpdate) {
    this.isEndOfUpdate = isEndOfUpdate;
  }


  /**
   * Return true if this ProtocolUpdateLedgerProofOrigin object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    ProtocolUpdateLedgerProofOrigin protocolUpdateLedgerProofOrigin = (ProtocolUpdateLedgerProofOrigin) o;
    return Objects.equals(this.protocolVersionName, protocolUpdateLedgerProofOrigin.protocolVersionName) &&
        Objects.equals(this.configHash, protocolUpdateLedgerProofOrigin.configHash) &&
        Objects.equals(this.batchGroupIdx, protocolUpdateLedgerProofOrigin.batchGroupIdx) &&
        Objects.equals(this.batchGroupName, protocolUpdateLedgerProofOrigin.batchGroupName) &&
        Objects.equals(this.batchIdx, protocolUpdateLedgerProofOrigin.batchIdx) &&
        Objects.equals(this.batchName, protocolUpdateLedgerProofOrigin.batchName) &&
        Objects.equals(this.isEndOfUpdate, protocolUpdateLedgerProofOrigin.isEndOfUpdate) &&
        super.equals(o);
  }

  @Override
  public int hashCode() {
    return Objects.hash(protocolVersionName, configHash, batchGroupIdx, batchGroupName, batchIdx, batchName, isEndOfUpdate, super.hashCode());
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class ProtocolUpdateLedgerProofOrigin {\n");
    sb.append("    ").append(toIndentedString(super.toString())).append("\n");
    sb.append("    protocolVersionName: ").append(toIndentedString(protocolVersionName)).append("\n");
    sb.append("    configHash: ").append(toIndentedString(configHash)).append("\n");
    sb.append("    batchGroupIdx: ").append(toIndentedString(batchGroupIdx)).append("\n");
    sb.append("    batchGroupName: ").append(toIndentedString(batchGroupName)).append("\n");
    sb.append("    batchIdx: ").append(toIndentedString(batchIdx)).append("\n");
    sb.append("    batchName: ").append(toIndentedString(batchName)).append("\n");
    sb.append("    isEndOfUpdate: ").append(toIndentedString(isEndOfUpdate)).append("\n");
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
  mappings.put("Consensus", ConsensusLedgerProofOrigin.class);
  mappings.put("Genesis", GenesisLedgerProofOrigin.class);
  mappings.put("ProtocolUpdate", ProtocolUpdateLedgerProofOrigin.class);
  mappings.put("ProtocolUpdateLedgerProofOrigin", ProtocolUpdateLedgerProofOrigin.class);
  JSON.registerDiscriminator(ProtocolUpdateLedgerProofOrigin.class, "type", mappings);
}
}

