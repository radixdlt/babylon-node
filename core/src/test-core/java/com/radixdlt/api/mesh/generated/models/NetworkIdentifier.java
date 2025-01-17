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
import com.radixdlt.api.mesh.generated.models.SubNetworkIdentifier;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * The network_identifier specifies which network a particular object is associated with. 
 */
@ApiModel(description = "The network_identifier specifies which network a particular object is associated with. ")
@JsonPropertyOrder({
  NetworkIdentifier.JSON_PROPERTY_BLOCKCHAIN,
  NetworkIdentifier.JSON_PROPERTY_NETWORK,
  NetworkIdentifier.JSON_PROPERTY_SUB_NETWORK_IDENTIFIER
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class NetworkIdentifier {
  public static final String JSON_PROPERTY_BLOCKCHAIN = "blockchain";
  private String blockchain;

  public static final String JSON_PROPERTY_NETWORK = "network";
  private String network;

  public static final String JSON_PROPERTY_SUB_NETWORK_IDENTIFIER = "sub_network_identifier";
  private SubNetworkIdentifier subNetworkIdentifier;

  public NetworkIdentifier() { 
  }

  public NetworkIdentifier blockchain(String blockchain) {
    this.blockchain = blockchain;
    return this;
  }

   /**
   * Get blockchain
   * @return blockchain
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(example = "bitcoin", required = true, value = "")
  @JsonProperty(JSON_PROPERTY_BLOCKCHAIN)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getBlockchain() {
    return blockchain;
  }


  @JsonProperty(JSON_PROPERTY_BLOCKCHAIN)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setBlockchain(String blockchain) {
    this.blockchain = blockchain;
  }


  public NetworkIdentifier network(String network) {
    this.network = network;
    return this;
  }

   /**
   * If a blockchain has a specific chain-id or network identifier, it should go in this field. It is up to the client to determine which network-specific identifier is mainnet or testnet. 
   * @return network
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(example = "mainnet", required = true, value = "If a blockchain has a specific chain-id or network identifier, it should go in this field. It is up to the client to determine which network-specific identifier is mainnet or testnet. ")
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


  public NetworkIdentifier subNetworkIdentifier(SubNetworkIdentifier subNetworkIdentifier) {
    this.subNetworkIdentifier = subNetworkIdentifier;
    return this;
  }

   /**
   * Get subNetworkIdentifier
   * @return subNetworkIdentifier
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "")
  @JsonProperty(JSON_PROPERTY_SUB_NETWORK_IDENTIFIER)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public SubNetworkIdentifier getSubNetworkIdentifier() {
    return subNetworkIdentifier;
  }


  @JsonProperty(JSON_PROPERTY_SUB_NETWORK_IDENTIFIER)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setSubNetworkIdentifier(SubNetworkIdentifier subNetworkIdentifier) {
    this.subNetworkIdentifier = subNetworkIdentifier;
  }


  /**
   * Return true if this NetworkIdentifier object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    NetworkIdentifier networkIdentifier = (NetworkIdentifier) o;
    return Objects.equals(this.blockchain, networkIdentifier.blockchain) &&
        Objects.equals(this.network, networkIdentifier.network) &&
        Objects.equals(this.subNetworkIdentifier, networkIdentifier.subNetworkIdentifier);
  }

  @Override
  public int hashCode() {
    return Objects.hash(blockchain, network, subNetworkIdentifier);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class NetworkIdentifier {\n");
    sb.append("    blockchain: ").append(toIndentedString(blockchain)).append("\n");
    sb.append("    network: ").append(toIndentedString(network)).append("\n");
    sb.append("    subNetworkIdentifier: ").append(toIndentedString(subNetworkIdentifier)).append("\n");
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

