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
import com.fasterxml.jackson.annotation.JsonTypeName;
import com.fasterxml.jackson.annotation.JsonValue;
import com.radixdlt.api.system.generated.models.ProtocolUpdateTrigger;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import java.util.ArrayList;
import java.util.List;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * ProtocolConfiguration
 */
@JsonPropertyOrder({
  ProtocolConfiguration.JSON_PROPERTY_GENESIS_PROTOCOL_VERSION,
  ProtocolConfiguration.JSON_PROPERTY_PROTOCOL_UPDATE_TRIGGERS
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class ProtocolConfiguration {
  public static final String JSON_PROPERTY_GENESIS_PROTOCOL_VERSION = "genesis_protocol_version";
  private String genesisProtocolVersion;

  public static final String JSON_PROPERTY_PROTOCOL_UPDATE_TRIGGERS = "protocol_update_triggers";
  private List<ProtocolUpdateTrigger> protocolUpdateTriggers = new ArrayList<>();


  public ProtocolConfiguration genesisProtocolVersion(String genesisProtocolVersion) {
    this.genesisProtocolVersion = genesisProtocolVersion;
    return this;
  }

   /**
   * Get genesisProtocolVersion
   * @return genesisProtocolVersion
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_GENESIS_PROTOCOL_VERSION)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getGenesisProtocolVersion() {
    return genesisProtocolVersion;
  }


  @JsonProperty(JSON_PROPERTY_GENESIS_PROTOCOL_VERSION)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setGenesisProtocolVersion(String genesisProtocolVersion) {
    this.genesisProtocolVersion = genesisProtocolVersion;
  }


  public ProtocolConfiguration protocolUpdateTriggers(List<ProtocolUpdateTrigger> protocolUpdateTriggers) {
    this.protocolUpdateTriggers = protocolUpdateTriggers;
    return this;
  }

  public ProtocolConfiguration addProtocolUpdateTriggersItem(ProtocolUpdateTrigger protocolUpdateTriggersItem) {
    this.protocolUpdateTriggers.add(protocolUpdateTriggersItem);
    return this;
  }

   /**
   * Get protocolUpdateTriggers
   * @return protocolUpdateTriggers
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_PROTOCOL_UPDATE_TRIGGERS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public List<ProtocolUpdateTrigger> getProtocolUpdateTriggers() {
    return protocolUpdateTriggers;
  }


  @JsonProperty(JSON_PROPERTY_PROTOCOL_UPDATE_TRIGGERS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setProtocolUpdateTriggers(List<ProtocolUpdateTrigger> protocolUpdateTriggers) {
    this.protocolUpdateTriggers = protocolUpdateTriggers;
  }


  /**
   * Return true if this ProtocolConfiguration object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    ProtocolConfiguration protocolConfiguration = (ProtocolConfiguration) o;
    return Objects.equals(this.genesisProtocolVersion, protocolConfiguration.genesisProtocolVersion) &&
        Objects.equals(this.protocolUpdateTriggers, protocolConfiguration.protocolUpdateTriggers);
  }

  @Override
  public int hashCode() {
    return Objects.hash(genesisProtocolVersion, protocolUpdateTriggers);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class ProtocolConfiguration {\n");
    sb.append("    genesisProtocolVersion: ").append(toIndentedString(genesisProtocolVersion)).append("\n");
    sb.append("    protocolUpdateTriggers: ").append(toIndentedString(protocolUpdateTriggers)).append("\n");
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

