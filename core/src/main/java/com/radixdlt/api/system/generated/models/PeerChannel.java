/*
 * Radix System API
 * This API is exposed by the Babylon Radix node to give clients access to information about the node itself, its configuration, status and subsystems.  It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Heavy load may impact the node's function.  If you require queries against ledger state, you may also wish to consider using the [Core API or Gateway API instead](https://docs-babylon.radixdlt.com/main/apis/api-specification.html). 
 *
 * The version of the OpenAPI document: v1.2.2
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
import com.radixdlt.api.system.generated.models.PeerApplicationVersion;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * PeerChannel
 */
@JsonPropertyOrder({
  PeerChannel.JSON_PROPERTY_TYPE,
  PeerChannel.JSON_PROPERTY_LOCAL_PORT,
  PeerChannel.JSON_PROPERTY_IP,
  PeerChannel.JSON_PROPERTY_URI,
  PeerChannel.JSON_PROPERTY_APPLICATION_VERSION
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class PeerChannel {
  /**
   * Gets or Sets type
   */
  public enum TypeEnum {
    IN("IN"),
    
    OUT("OUT");

    private String value;

    TypeEnum(String value) {
      this.value = value;
    }

    @JsonValue
    public String getValue() {
      return value;
    }

    @Override
    public String toString() {
      return String.valueOf(value);
    }

    @JsonCreator
    public static TypeEnum fromValue(String value) {
      for (TypeEnum b : TypeEnum.values()) {
        if (b.value.equals(value)) {
          return b;
        }
      }
      throw new IllegalArgumentException("Unexpected value '" + value + "'");
    }
  }

  public static final String JSON_PROPERTY_TYPE = "type";
  private TypeEnum type;

  public static final String JSON_PROPERTY_LOCAL_PORT = "local_port";
  private Integer localPort;

  public static final String JSON_PROPERTY_IP = "ip";
  private String ip;

  public static final String JSON_PROPERTY_URI = "uri";
  private String uri;

  public static final String JSON_PROPERTY_APPLICATION_VERSION = "application_version";
  private PeerApplicationVersion applicationVersion;


  public PeerChannel type(TypeEnum type) {
    this.type = type;
    return this;
  }

   /**
   * Get type
   * @return type
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_TYPE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public TypeEnum getType() {
    return type;
  }


  @JsonProperty(JSON_PROPERTY_TYPE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setType(TypeEnum type) {
    this.type = type;
  }


  public PeerChannel localPort(Integer localPort) {
    this.localPort = localPort;
    return this;
  }

   /**
   * Get localPort
   * @return localPort
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_LOCAL_PORT)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Integer getLocalPort() {
    return localPort;
  }


  @JsonProperty(JSON_PROPERTY_LOCAL_PORT)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setLocalPort(Integer localPort) {
    this.localPort = localPort;
  }


  public PeerChannel ip(String ip) {
    this.ip = ip;
    return this;
  }

   /**
   * Get ip
   * @return ip
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_IP)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getIp() {
    return ip;
  }


  @JsonProperty(JSON_PROPERTY_IP)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setIp(String ip) {
    this.ip = ip;
  }


  public PeerChannel uri(String uri) {
    this.uri = uri;
    return this;
  }

   /**
   * Get uri
   * @return uri
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "")
  @JsonProperty(JSON_PROPERTY_URI)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public String getUri() {
    return uri;
  }


  @JsonProperty(JSON_PROPERTY_URI)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setUri(String uri) {
    this.uri = uri;
  }


  public PeerChannel applicationVersion(PeerApplicationVersion applicationVersion) {
    this.applicationVersion = applicationVersion;
    return this;
  }

   /**
   * Get applicationVersion
   * @return applicationVersion
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "")
  @JsonProperty(JSON_PROPERTY_APPLICATION_VERSION)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public PeerApplicationVersion getApplicationVersion() {
    return applicationVersion;
  }


  @JsonProperty(JSON_PROPERTY_APPLICATION_VERSION)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setApplicationVersion(PeerApplicationVersion applicationVersion) {
    this.applicationVersion = applicationVersion;
  }


  /**
   * Return true if this PeerChannel object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    PeerChannel peerChannel = (PeerChannel) o;
    return Objects.equals(this.type, peerChannel.type) &&
        Objects.equals(this.localPort, peerChannel.localPort) &&
        Objects.equals(this.ip, peerChannel.ip) &&
        Objects.equals(this.uri, peerChannel.uri) &&
        Objects.equals(this.applicationVersion, peerChannel.applicationVersion);
  }

  @Override
  public int hashCode() {
    return Objects.hash(type, localPort, ip, uri, applicationVersion);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class PeerChannel {\n");
    sb.append("    type: ").append(toIndentedString(type)).append("\n");
    sb.append("    localPort: ").append(toIndentedString(localPort)).append("\n");
    sb.append("    ip: ").append(toIndentedString(ip)).append("\n");
    sb.append("    uri: ").append(toIndentedString(uri)).append("\n");
    sb.append("    applicationVersion: ").append(toIndentedString(applicationVersion)).append("\n");
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

