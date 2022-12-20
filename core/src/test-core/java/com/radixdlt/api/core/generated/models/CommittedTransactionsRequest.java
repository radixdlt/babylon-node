/*
 * Babylon Core API
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node. It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Heavy load may impact the node's function.  If you require queries against historical ledger state, you may also wish to consider using the [Gateway API](https://betanet-gateway.redoc.ly/). 
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
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * A request to retrieve a sublist of committed transactions from the ledger. 
 */
@ApiModel(description = "A request to retrieve a sublist of committed transactions from the ledger. ")
@JsonPropertyOrder({
  CommittedTransactionsRequest.JSON_PROPERTY_NETWORK,
  CommittedTransactionsRequest.JSON_PROPERTY_FROM_STATE_VERSION,
  CommittedTransactionsRequest.JSON_PROPERTY_LIMIT
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class CommittedTransactionsRequest {
  public static final String JSON_PROPERTY_NETWORK = "network";
  private String network;

  public static final String JSON_PROPERTY_FROM_STATE_VERSION = "from_state_version";
  private Long fromStateVersion;

  public static final String JSON_PROPERTY_LIMIT = "limit";
  private Integer limit;

  public CommittedTransactionsRequest() { 
  }

  public CommittedTransactionsRequest network(String network) {
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


  public CommittedTransactionsRequest fromStateVersion(Long fromStateVersion) {
    this.fromStateVersion = fromStateVersion;
    return this;
  }

   /**
   * An integer between &#x60;1&#x60; and &#x60;10^13&#x60;, giving the first (resultant) state version to be returned
   * minimum: 1
   * maximum: 100000000000000
   * @return fromStateVersion
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "An integer between `1` and `10^13`, giving the first (resultant) state version to be returned")
  @JsonProperty(JSON_PROPERTY_FROM_STATE_VERSION)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Long getFromStateVersion() {
    return fromStateVersion;
  }


  @JsonProperty(JSON_PROPERTY_FROM_STATE_VERSION)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setFromStateVersion(Long fromStateVersion) {
    this.fromStateVersion = fromStateVersion;
  }


  public CommittedTransactionsRequest limit(Integer limit) {
    this.limit = limit;
    return this;
  }

   /**
   * The maximum number of transactions that will be returned.
   * @return limit
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The maximum number of transactions that will be returned.")
  @JsonProperty(JSON_PROPERTY_LIMIT)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Integer getLimit() {
    return limit;
  }


  @JsonProperty(JSON_PROPERTY_LIMIT)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setLimit(Integer limit) {
    this.limit = limit;
  }


  /**
   * Return true if this CommittedTransactionsRequest object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    CommittedTransactionsRequest committedTransactionsRequest = (CommittedTransactionsRequest) o;
    return Objects.equals(this.network, committedTransactionsRequest.network) &&
        Objects.equals(this.fromStateVersion, committedTransactionsRequest.fromStateVersion) &&
        Objects.equals(this.limit, committedTransactionsRequest.limit);
  }

  @Override
  public int hashCode() {
    return Objects.hash(network, fromStateVersion, limit);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class CommittedTransactionsRequest {\n");
    sb.append("    network: ").append(toIndentedString(network)).append("\n");
    sb.append("    fromStateVersion: ").append(toIndentedString(fromStateVersion)).append("\n");
    sb.append("    limit: ").append(toIndentedString(limit)).append("\n");
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

