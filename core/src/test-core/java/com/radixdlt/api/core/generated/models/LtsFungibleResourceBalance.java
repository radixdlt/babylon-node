/*
 * Babylon Core API
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node. It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Heavy load may impact the node's function.  If you require queries against historical ledger state, you may also wish to consider using the [Gateway API](https://betanet-gateway.redoc.ly/). 
 *
 * The version of the OpenAPI document: 0.3.0
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
 * LtsFungibleResourceBalance
 */
@JsonPropertyOrder({
  LtsFungibleResourceBalance.JSON_PROPERTY_FUNGIBLE_RESOURCE_ADDRESS,
  LtsFungibleResourceBalance.JSON_PROPERTY_AMOUNT
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class LtsFungibleResourceBalance {
  public static final String JSON_PROPERTY_FUNGIBLE_RESOURCE_ADDRESS = "fungible_resource_address";
  private String fungibleResourceAddress;

  public static final String JSON_PROPERTY_AMOUNT = "amount";
  private String amount;

  public LtsFungibleResourceBalance() { 
  }

  public LtsFungibleResourceBalance fungibleResourceAddress(String fungibleResourceAddress) {
    this.fungibleResourceAddress = fungibleResourceAddress;
    return this;
  }

   /**
   * The Bech32m-encoded human readable version of the fungible resource&#39;s global address
   * @return fungibleResourceAddress
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The Bech32m-encoded human readable version of the fungible resource's global address")
  @JsonProperty(JSON_PROPERTY_FUNGIBLE_RESOURCE_ADDRESS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getFungibleResourceAddress() {
    return fungibleResourceAddress;
  }


  @JsonProperty(JSON_PROPERTY_FUNGIBLE_RESOURCE_ADDRESS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setFungibleResourceAddress(String fungibleResourceAddress) {
    this.fungibleResourceAddress = fungibleResourceAddress;
  }


  public LtsFungibleResourceBalance amount(String amount) {
    this.amount = amount;
    return this;
  }

   /**
   * The string-encoded decimal representing the amount of the fungible resource. A decimal is formed of some signed integer &#x60;m&#x60; of attos (&#x60;10^(-18)&#x60;) units, where &#x60;-2^(256 - 1) &lt;&#x3D; m &lt; 2^(256 - 1)&#x60;. 
   * @return amount
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The string-encoded decimal representing the amount of the fungible resource. A decimal is formed of some signed integer `m` of attos (`10^(-18)`) units, where `-2^(256 - 1) <= m < 2^(256 - 1)`. ")
  @JsonProperty(JSON_PROPERTY_AMOUNT)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getAmount() {
    return amount;
  }


  @JsonProperty(JSON_PROPERTY_AMOUNT)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setAmount(String amount) {
    this.amount = amount;
  }


  /**
   * Return true if this LtsFungibleResourceBalance object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    LtsFungibleResourceBalance ltsFungibleResourceBalance = (LtsFungibleResourceBalance) o;
    return Objects.equals(this.fungibleResourceAddress, ltsFungibleResourceBalance.fungibleResourceAddress) &&
        Objects.equals(this.amount, ltsFungibleResourceBalance.amount);
  }

  @Override
  public int hashCode() {
    return Objects.hash(fungibleResourceAddress, amount);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class LtsFungibleResourceBalance {\n");
    sb.append("    fungibleResourceAddress: ").append(toIndentedString(fungibleResourceAddress)).append("\n");
    sb.append("    amount: ").append(toIndentedString(amount)).append("\n");
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

