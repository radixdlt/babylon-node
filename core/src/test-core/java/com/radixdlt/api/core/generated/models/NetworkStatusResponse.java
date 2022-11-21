/*
 * Babylon Core API
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 0.1.0
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
import com.radixdlt.api.core.generated.models.CommittedStateIdentifier;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * NetworkStatusResponse
 */
@JsonPropertyOrder({
  NetworkStatusResponse.JSON_PROPERTY_PRE_GENESIS_STATE_IDENTIFIER,
  NetworkStatusResponse.JSON_PROPERTY_POST_GENESIS_STATE_IDENTIFIER,
  NetworkStatusResponse.JSON_PROPERTY_CURRENT_STATE_IDENTIFIER
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class NetworkStatusResponse {
  public static final String JSON_PROPERTY_PRE_GENESIS_STATE_IDENTIFIER = "pre_genesis_state_identifier";
  private CommittedStateIdentifier preGenesisStateIdentifier;

  public static final String JSON_PROPERTY_POST_GENESIS_STATE_IDENTIFIER = "post_genesis_state_identifier";
  private CommittedStateIdentifier postGenesisStateIdentifier;

  public static final String JSON_PROPERTY_CURRENT_STATE_IDENTIFIER = "current_state_identifier";
  private CommittedStateIdentifier currentStateIdentifier;

  public NetworkStatusResponse() { 
  }

  public NetworkStatusResponse preGenesisStateIdentifier(CommittedStateIdentifier preGenesisStateIdentifier) {
    this.preGenesisStateIdentifier = preGenesisStateIdentifier;
    return this;
  }

   /**
   * Get preGenesisStateIdentifier
   * @return preGenesisStateIdentifier
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_PRE_GENESIS_STATE_IDENTIFIER)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public CommittedStateIdentifier getPreGenesisStateIdentifier() {
    return preGenesisStateIdentifier;
  }


  @JsonProperty(JSON_PROPERTY_PRE_GENESIS_STATE_IDENTIFIER)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setPreGenesisStateIdentifier(CommittedStateIdentifier preGenesisStateIdentifier) {
    this.preGenesisStateIdentifier = preGenesisStateIdentifier;
  }


  public NetworkStatusResponse postGenesisStateIdentifier(CommittedStateIdentifier postGenesisStateIdentifier) {
    this.postGenesisStateIdentifier = postGenesisStateIdentifier;
    return this;
  }

   /**
   * Get postGenesisStateIdentifier
   * @return postGenesisStateIdentifier
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "")
  @JsonProperty(JSON_PROPERTY_POST_GENESIS_STATE_IDENTIFIER)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public CommittedStateIdentifier getPostGenesisStateIdentifier() {
    return postGenesisStateIdentifier;
  }


  @JsonProperty(JSON_PROPERTY_POST_GENESIS_STATE_IDENTIFIER)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setPostGenesisStateIdentifier(CommittedStateIdentifier postGenesisStateIdentifier) {
    this.postGenesisStateIdentifier = postGenesisStateIdentifier;
  }


  public NetworkStatusResponse currentStateIdentifier(CommittedStateIdentifier currentStateIdentifier) {
    this.currentStateIdentifier = currentStateIdentifier;
    return this;
  }

   /**
   * Get currentStateIdentifier
   * @return currentStateIdentifier
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_CURRENT_STATE_IDENTIFIER)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public CommittedStateIdentifier getCurrentStateIdentifier() {
    return currentStateIdentifier;
  }


  @JsonProperty(JSON_PROPERTY_CURRENT_STATE_IDENTIFIER)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setCurrentStateIdentifier(CommittedStateIdentifier currentStateIdentifier) {
    this.currentStateIdentifier = currentStateIdentifier;
  }


  /**
   * Return true if this NetworkStatusResponse object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    NetworkStatusResponse networkStatusResponse = (NetworkStatusResponse) o;
    return Objects.equals(this.preGenesisStateIdentifier, networkStatusResponse.preGenesisStateIdentifier) &&
        Objects.equals(this.postGenesisStateIdentifier, networkStatusResponse.postGenesisStateIdentifier) &&
        Objects.equals(this.currentStateIdentifier, networkStatusResponse.currentStateIdentifier);
  }

  @Override
  public int hashCode() {
    return Objects.hash(preGenesisStateIdentifier, postGenesisStateIdentifier, currentStateIdentifier);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class NetworkStatusResponse {\n");
    sb.append("    preGenesisStateIdentifier: ").append(toIndentedString(preGenesisStateIdentifier)).append("\n");
    sb.append("    postGenesisStateIdentifier: ").append(toIndentedString(postGenesisStateIdentifier)).append("\n");
    sb.append("    currentStateIdentifier: ").append(toIndentedString(currentStateIdentifier)).append("\n");
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

