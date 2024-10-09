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
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonTypeName;
import com.fasterxml.jackson.annotation.JsonValue;
import com.radixdlt.api.core.generated.models.AddressType;
import com.radixdlt.api.core.generated.models.NetworkConfigurationResponseVersion;
import com.radixdlt.api.core.generated.models.NetworkConfigurationResponseWellKnownAddresses;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import java.util.ArrayList;
import java.util.List;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * NetworkConfigurationResponse
 */
@JsonPropertyOrder({
  NetworkConfigurationResponse.JSON_PROPERTY_VERSION,
  NetworkConfigurationResponse.JSON_PROPERTY_NETWORK,
  NetworkConfigurationResponse.JSON_PROPERTY_NETWORK_ID,
  NetworkConfigurationResponse.JSON_PROPERTY_NETWORK_HRP_SUFFIX,
  NetworkConfigurationResponse.JSON_PROPERTY_USD_PRICE_IN_XRD,
  NetworkConfigurationResponse.JSON_PROPERTY_ADDRESS_TYPES,
  NetworkConfigurationResponse.JSON_PROPERTY_WELL_KNOWN_ADDRESSES
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class NetworkConfigurationResponse {
  public static final String JSON_PROPERTY_VERSION = "version";
  private NetworkConfigurationResponseVersion version;

  public static final String JSON_PROPERTY_NETWORK = "network";
  private String network;

  public static final String JSON_PROPERTY_NETWORK_ID = "network_id";
  private Integer networkId;

  public static final String JSON_PROPERTY_NETWORK_HRP_SUFFIX = "network_hrp_suffix";
  private String networkHrpSuffix;

  public static final String JSON_PROPERTY_USD_PRICE_IN_XRD = "usd_price_in_xrd";
  private String usdPriceInXrd;

  public static final String JSON_PROPERTY_ADDRESS_TYPES = "address_types";
  private List<AddressType> addressTypes = new ArrayList<>();

  public static final String JSON_PROPERTY_WELL_KNOWN_ADDRESSES = "well_known_addresses";
  private NetworkConfigurationResponseWellKnownAddresses wellKnownAddresses;

  public NetworkConfigurationResponse() { 
  }

  public NetworkConfigurationResponse version(NetworkConfigurationResponseVersion version) {
    this.version = version;
    return this;
  }

   /**
   * Get version
   * @return version
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_VERSION)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public NetworkConfigurationResponseVersion getVersion() {
    return version;
  }


  @JsonProperty(JSON_PROPERTY_VERSION)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setVersion(NetworkConfigurationResponseVersion version) {
    this.version = version;
  }


  public NetworkConfigurationResponse network(String network) {
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


  public NetworkConfigurationResponse networkId(Integer networkId) {
    this.networkId = networkId;
    return this;
  }

   /**
   * The logical id of the network
   * minimum: 0
   * maximum: 255
   * @return networkId
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The logical id of the network")
  @JsonProperty(JSON_PROPERTY_NETWORK_ID)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Integer getNetworkId() {
    return networkId;
  }


  @JsonProperty(JSON_PROPERTY_NETWORK_ID)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setNetworkId(Integer networkId) {
    this.networkId = networkId;
  }


  public NetworkConfigurationResponse networkHrpSuffix(String networkHrpSuffix) {
    this.networkHrpSuffix = networkHrpSuffix;
    return this;
  }

   /**
   * The network suffix used for Bech32m HRPs used for addressing.
   * @return networkHrpSuffix
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The network suffix used for Bech32m HRPs used for addressing.")
  @JsonProperty(JSON_PROPERTY_NETWORK_HRP_SUFFIX)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getNetworkHrpSuffix() {
    return networkHrpSuffix;
  }


  @JsonProperty(JSON_PROPERTY_NETWORK_HRP_SUFFIX)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setNetworkHrpSuffix(String networkHrpSuffix) {
    this.networkHrpSuffix = networkHrpSuffix;
  }


  public NetworkConfigurationResponse usdPriceInXrd(String usdPriceInXrd) {
    this.usdPriceInXrd = usdPriceInXrd;
    return this;
  }

   /**
   * The current value of the protocol-based USD/XRD multiplier (i.e. an amount of XRDs to be paid for 1 USD). A decimal is formed of some signed integer &#x60;m&#x60; of attos (&#x60;10^(-18)&#x60;) units, where &#x60;-2^(192 - 1) &lt;&#x3D; m &lt; 2^(192 - 1)&#x60;. 
   * @return usdPriceInXrd
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The current value of the protocol-based USD/XRD multiplier (i.e. an amount of XRDs to be paid for 1 USD). A decimal is formed of some signed integer `m` of attos (`10^(-18)`) units, where `-2^(192 - 1) <= m < 2^(192 - 1)`. ")
  @JsonProperty(JSON_PROPERTY_USD_PRICE_IN_XRD)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getUsdPriceInXrd() {
    return usdPriceInXrd;
  }


  @JsonProperty(JSON_PROPERTY_USD_PRICE_IN_XRD)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setUsdPriceInXrd(String usdPriceInXrd) {
    this.usdPriceInXrd = usdPriceInXrd;
  }


  public NetworkConfigurationResponse addressTypes(List<AddressType> addressTypes) {
    this.addressTypes = addressTypes;
    return this;
  }

  public NetworkConfigurationResponse addAddressTypesItem(AddressType addressTypesItem) {
    this.addressTypes.add(addressTypesItem);
    return this;
  }

   /**
   * Get addressTypes
   * @return addressTypes
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_ADDRESS_TYPES)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public List<AddressType> getAddressTypes() {
    return addressTypes;
  }


  @JsonProperty(JSON_PROPERTY_ADDRESS_TYPES)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setAddressTypes(List<AddressType> addressTypes) {
    this.addressTypes = addressTypes;
  }


  public NetworkConfigurationResponse wellKnownAddresses(NetworkConfigurationResponseWellKnownAddresses wellKnownAddresses) {
    this.wellKnownAddresses = wellKnownAddresses;
    return this;
  }

   /**
   * Get wellKnownAddresses
   * @return wellKnownAddresses
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_WELL_KNOWN_ADDRESSES)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public NetworkConfigurationResponseWellKnownAddresses getWellKnownAddresses() {
    return wellKnownAddresses;
  }


  @JsonProperty(JSON_PROPERTY_WELL_KNOWN_ADDRESSES)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setWellKnownAddresses(NetworkConfigurationResponseWellKnownAddresses wellKnownAddresses) {
    this.wellKnownAddresses = wellKnownAddresses;
  }


  /**
   * Return true if this NetworkConfigurationResponse object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    NetworkConfigurationResponse networkConfigurationResponse = (NetworkConfigurationResponse) o;
    return Objects.equals(this.version, networkConfigurationResponse.version) &&
        Objects.equals(this.network, networkConfigurationResponse.network) &&
        Objects.equals(this.networkId, networkConfigurationResponse.networkId) &&
        Objects.equals(this.networkHrpSuffix, networkConfigurationResponse.networkHrpSuffix) &&
        Objects.equals(this.usdPriceInXrd, networkConfigurationResponse.usdPriceInXrd) &&
        Objects.equals(this.addressTypes, networkConfigurationResponse.addressTypes) &&
        Objects.equals(this.wellKnownAddresses, networkConfigurationResponse.wellKnownAddresses);
  }

  @Override
  public int hashCode() {
    return Objects.hash(version, network, networkId, networkHrpSuffix, usdPriceInXrd, addressTypes, wellKnownAddresses);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class NetworkConfigurationResponse {\n");
    sb.append("    version: ").append(toIndentedString(version)).append("\n");
    sb.append("    network: ").append(toIndentedString(network)).append("\n");
    sb.append("    networkId: ").append(toIndentedString(networkId)).append("\n");
    sb.append("    networkHrpSuffix: ").append(toIndentedString(networkHrpSuffix)).append("\n");
    sb.append("    usdPriceInXrd: ").append(toIndentedString(usdPriceInXrd)).append("\n");
    sb.append("    addressTypes: ").append(toIndentedString(addressTypes)).append("\n");
    sb.append("    wellKnownAddresses: ").append(toIndentedString(wellKnownAddresses)).append("\n");
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

