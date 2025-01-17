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
import com.radixdlt.api.mesh.generated.models.CoinAction;
import com.radixdlt.api.mesh.generated.models.CoinIdentifier;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * CoinChange is used to represent a change in state of a some coin identified by a coin_identifier. This object is part of the Operation model and must be populated for UTXO-based blockchains.  Coincidentally, this abstraction of UTXOs allows for supporting both account-based transfers and UTXO-based transfers on the same blockchain (when a transfer is account-based, don&#39;t populate this model). 
 */
@ApiModel(description = "CoinChange is used to represent a change in state of a some coin identified by a coin_identifier. This object is part of the Operation model and must be populated for UTXO-based blockchains.  Coincidentally, this abstraction of UTXOs allows for supporting both account-based transfers and UTXO-based transfers on the same blockchain (when a transfer is account-based, don't populate this model). ")
@JsonPropertyOrder({
  CoinChange.JSON_PROPERTY_COIN_IDENTIFIER,
  CoinChange.JSON_PROPERTY_COIN_ACTION
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class CoinChange {
  public static final String JSON_PROPERTY_COIN_IDENTIFIER = "coin_identifier";
  private CoinIdentifier coinIdentifier;

  public static final String JSON_PROPERTY_COIN_ACTION = "coin_action";
  private CoinAction coinAction;

  public CoinChange() { 
  }

  public CoinChange coinIdentifier(CoinIdentifier coinIdentifier) {
    this.coinIdentifier = coinIdentifier;
    return this;
  }

   /**
   * Get coinIdentifier
   * @return coinIdentifier
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_COIN_IDENTIFIER)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public CoinIdentifier getCoinIdentifier() {
    return coinIdentifier;
  }


  @JsonProperty(JSON_PROPERTY_COIN_IDENTIFIER)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setCoinIdentifier(CoinIdentifier coinIdentifier) {
    this.coinIdentifier = coinIdentifier;
  }


  public CoinChange coinAction(CoinAction coinAction) {
    this.coinAction = coinAction;
    return this;
  }

   /**
   * Get coinAction
   * @return coinAction
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_COIN_ACTION)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public CoinAction getCoinAction() {
    return coinAction;
  }


  @JsonProperty(JSON_PROPERTY_COIN_ACTION)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setCoinAction(CoinAction coinAction) {
    this.coinAction = coinAction;
  }


  /**
   * Return true if this CoinChange object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    CoinChange coinChange = (CoinChange) o;
    return Objects.equals(this.coinIdentifier, coinChange.coinIdentifier) &&
        Objects.equals(this.coinAction, coinChange.coinAction);
  }

  @Override
  public int hashCode() {
    return Objects.hash(coinIdentifier, coinAction);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class CoinChange {\n");
    sb.append("    coinIdentifier: ").append(toIndentedString(coinIdentifier)).append("\n");
    sb.append("    coinAction: ").append(toIndentedString(coinAction)).append("\n");
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

