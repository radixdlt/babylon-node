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
 * Currency is composed of a canonical Symbol and Decimals. This Decimals value is used to convert an Amount.Value from atomic units (Satoshis) to standard units (Bitcoins). 
 */
@ApiModel(description = "Currency is composed of a canonical Symbol and Decimals. This Decimals value is used to convert an Amount.Value from atomic units (Satoshis) to standard units (Bitcoins). ")
@JsonPropertyOrder({
  Currency.JSON_PROPERTY_SYMBOL,
  Currency.JSON_PROPERTY_DECIMALS,
  Currency.JSON_PROPERTY_METADATA
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class Currency {
  public static final String JSON_PROPERTY_SYMBOL = "symbol";
  private String symbol;

  public static final String JSON_PROPERTY_DECIMALS = "decimals";
  private Integer decimals;

  public static final String JSON_PROPERTY_METADATA = "metadata";
  private Object metadata;

  public Currency() { 
  }

  public Currency symbol(String symbol) {
    this.symbol = symbol;
    return this;
  }

   /**
   * Canonical symbol associated with a currency. 
   * @return symbol
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(example = "BTC", required = true, value = "Canonical symbol associated with a currency. ")
  @JsonProperty(JSON_PROPERTY_SYMBOL)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getSymbol() {
    return symbol;
  }


  @JsonProperty(JSON_PROPERTY_SYMBOL)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setSymbol(String symbol) {
    this.symbol = symbol;
  }


  public Currency decimals(Integer decimals) {
    this.decimals = decimals;
    return this;
  }

   /**
   * Number of decimal places in the standard unit representation of the amount.  For example, BTC has 8 decimals. Note that it is not possible to represent the value of some currency in atomic units that is not base 10. 
   * minimum: 0
   * @return decimals
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(example = "8", required = true, value = "Number of decimal places in the standard unit representation of the amount.  For example, BTC has 8 decimals. Note that it is not possible to represent the value of some currency in atomic units that is not base 10. ")
  @JsonProperty(JSON_PROPERTY_DECIMALS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Integer getDecimals() {
    return decimals;
  }


  @JsonProperty(JSON_PROPERTY_DECIMALS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setDecimals(Integer decimals) {
    this.decimals = decimals;
  }


  public Currency metadata(Object metadata) {
    this.metadata = metadata;
    return this;
  }

   /**
   * Any additional information related to the currency itself.  For example, it would be useful to populate this object with the contract address of an ERC-20 token. 
   * @return metadata
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(example = "{Issuer=Satoshi}", value = "Any additional information related to the currency itself.  For example, it would be useful to populate this object with the contract address of an ERC-20 token. ")
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
   * Return true if this Currency object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    Currency currency = (Currency) o;
    return Objects.equals(this.symbol, currency.symbol) &&
        Objects.equals(this.decimals, currency.decimals) &&
        Objects.equals(this.metadata, currency.metadata);
  }

  @Override
  public int hashCode() {
    return Objects.hash(symbol, decimals, metadata);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class Currency {\n");
    sb.append("    symbol: ").append(toIndentedString(symbol)).append("\n");
    sb.append("    decimals: ").append(toIndentedString(decimals)).append("\n");
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

