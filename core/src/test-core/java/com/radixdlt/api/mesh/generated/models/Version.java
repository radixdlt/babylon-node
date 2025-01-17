/*
 * Rosetta
 * Build Once. Integrate Your Blockchain Everywhere. 
 *
 * The version of the OpenAPI document: 1.4.13
 * 
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */


package com.radixdlt.api.mesh.generated.models;

import java.util.Objects;
import java.util.Arrays;
import java.util.Map;
import java.util.HashMap;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonTypeName;
import com.fasterxml.jackson.annotation.JsonValue;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * The Version object is utilized to inform the client of the versions of different components of the Rosetta implementation. 
 */
@ApiModel(description = "The Version object is utilized to inform the client of the versions of different components of the Rosetta implementation. ")
@JsonPropertyOrder({
  Version.JSON_PROPERTY_ROSETTA_VERSION,
  Version.JSON_PROPERTY_NODE_VERSION,
  Version.JSON_PROPERTY_MIDDLEWARE_VERSION,
  Version.JSON_PROPERTY_METADATA
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class Version {
  public static final String JSON_PROPERTY_ROSETTA_VERSION = "rosetta_version";
  private String rosettaVersion;

  public static final String JSON_PROPERTY_NODE_VERSION = "node_version";
  private String nodeVersion;

  public static final String JSON_PROPERTY_MIDDLEWARE_VERSION = "middleware_version";
  private String middlewareVersion;

  public static final String JSON_PROPERTY_METADATA = "metadata";
  private Object metadata;

  public Version() { 
  }

  public Version rosettaVersion(String rosettaVersion) {
    this.rosettaVersion = rosettaVersion;
    return this;
  }

   /**
   * The rosetta_version is the version of the Rosetta interface the implementation adheres to. This can be useful for clients looking to reliably parse responses. 
   * @return rosettaVersion
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(example = "1.2.5", required = true, value = "The rosetta_version is the version of the Rosetta interface the implementation adheres to. This can be useful for clients looking to reliably parse responses. ")
  @JsonProperty(JSON_PROPERTY_ROSETTA_VERSION)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getRosettaVersion() {
    return rosettaVersion;
  }


  @JsonProperty(JSON_PROPERTY_ROSETTA_VERSION)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setRosettaVersion(String rosettaVersion) {
    this.rosettaVersion = rosettaVersion;
  }


  public Version nodeVersion(String nodeVersion) {
    this.nodeVersion = nodeVersion;
    return this;
  }

   /**
   * The node_version is the canonical version of the node runtime. This can help clients manage deployments. 
   * @return nodeVersion
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(example = "1.0.2", required = true, value = "The node_version is the canonical version of the node runtime. This can help clients manage deployments. ")
  @JsonProperty(JSON_PROPERTY_NODE_VERSION)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getNodeVersion() {
    return nodeVersion;
  }


  @JsonProperty(JSON_PROPERTY_NODE_VERSION)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setNodeVersion(String nodeVersion) {
    this.nodeVersion = nodeVersion;
  }


  public Version middlewareVersion(String middlewareVersion) {
    this.middlewareVersion = middlewareVersion;
    return this;
  }

   /**
   * When a middleware server is used to adhere to the Rosetta interface, it should return its version here. This can help clients manage deployments. 
   * @return middlewareVersion
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(example = "0.2.7", value = "When a middleware server is used to adhere to the Rosetta interface, it should return its version here. This can help clients manage deployments. ")
  @JsonProperty(JSON_PROPERTY_MIDDLEWARE_VERSION)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public String getMiddlewareVersion() {
    return middlewareVersion;
  }


  @JsonProperty(JSON_PROPERTY_MIDDLEWARE_VERSION)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setMiddlewareVersion(String middlewareVersion) {
    this.middlewareVersion = middlewareVersion;
  }


  public Version metadata(Object metadata) {
    this.metadata = metadata;
    return this;
  }

   /**
   * Any other information that may be useful about versioning of dependent services should be returned here. 
   * @return metadata
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "Any other information that may be useful about versioning of dependent services should be returned here. ")
  @JsonProperty(JSON_PROPERTY_METADATA)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public Object getMetadata() {
    return metadata;
  }


  @JsonProperty(JSON_PROPERTY_METADATA)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setMetadata(Object metadata) {
    this.metadata = metadata;
  }


  /**
   * Return true if this Version object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    Version version = (Version) o;
    return Objects.equals(this.rosettaVersion, version.rosettaVersion) &&
        Objects.equals(this.nodeVersion, version.nodeVersion) &&
        Objects.equals(this.middlewareVersion, version.middlewareVersion) &&
        Objects.equals(this.metadata, version.metadata);
  }

  @Override
  public int hashCode() {
    return Objects.hash(rosettaVersion, nodeVersion, middlewareVersion, metadata);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class Version {\n");
    sb.append("    rosettaVersion: ").append(toIndentedString(rosettaVersion)).append("\n");
    sb.append("    nodeVersion: ").append(toIndentedString(nodeVersion)).append("\n");
    sb.append("    middlewareVersion: ").append(toIndentedString(middlewareVersion)).append("\n");
    sb.append("    metadata: ").append(toIndentedString(metadata)).append("\n");
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

