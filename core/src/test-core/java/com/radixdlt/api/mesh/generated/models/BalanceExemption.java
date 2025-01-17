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
import com.radixdlt.api.mesh.generated.models.Currency;
import com.radixdlt.api.mesh.generated.models.ExemptionType;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * BalanceExemption indicates that the balance for an exempt account could change without a corresponding Operation. This typically occurs with staking rewards, vesting balances, and Currencies with a dynamic supply.  Currently, it is possible to exempt an account from strict reconciliation by SubAccountIdentifier.Address or by Currency. This means that any account with SubAccountIdentifier.Address would be exempt or any balance of a particular Currency would be exempt, respectively.  BalanceExemptions should be used sparingly as they may introduce significant complexity for integrators that attempt to reconcile all account balance changes.  If your implementation relies on any BalanceExemptions, you MUST implement historical balance lookup (the ability to query an account balance at any BlockIdentifier). 
 */
@ApiModel(description = "BalanceExemption indicates that the balance for an exempt account could change without a corresponding Operation. This typically occurs with staking rewards, vesting balances, and Currencies with a dynamic supply.  Currently, it is possible to exempt an account from strict reconciliation by SubAccountIdentifier.Address or by Currency. This means that any account with SubAccountIdentifier.Address would be exempt or any balance of a particular Currency would be exempt, respectively.  BalanceExemptions should be used sparingly as they may introduce significant complexity for integrators that attempt to reconcile all account balance changes.  If your implementation relies on any BalanceExemptions, you MUST implement historical balance lookup (the ability to query an account balance at any BlockIdentifier). ")
@JsonPropertyOrder({
  BalanceExemption.JSON_PROPERTY_SUB_ACCOUNT_ADDRESS,
  BalanceExemption.JSON_PROPERTY_CURRENCY,
  BalanceExemption.JSON_PROPERTY_EXEMPTION_TYPE
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class BalanceExemption {
  public static final String JSON_PROPERTY_SUB_ACCOUNT_ADDRESS = "sub_account_address";
  private String subAccountAddress;

  public static final String JSON_PROPERTY_CURRENCY = "currency";
  private Currency currency;

  public static final String JSON_PROPERTY_EXEMPTION_TYPE = "exemption_type";
  private ExemptionType exemptionType;

  public BalanceExemption() { 
  }

  public BalanceExemption subAccountAddress(String subAccountAddress) {
    this.subAccountAddress = subAccountAddress;
    return this;
  }

   /**
   * SubAccountAddress is the SubAccountIdentifier.Address that the BalanceExemption applies to (regardless of the value of SubAccountIdentifier.Metadata). 
   * @return subAccountAddress
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(example = "staking", value = "SubAccountAddress is the SubAccountIdentifier.Address that the BalanceExemption applies to (regardless of the value of SubAccountIdentifier.Metadata). ")
  @JsonProperty(JSON_PROPERTY_SUB_ACCOUNT_ADDRESS)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public String getSubAccountAddress() {
    return subAccountAddress;
  }


  @JsonProperty(JSON_PROPERTY_SUB_ACCOUNT_ADDRESS)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setSubAccountAddress(String subAccountAddress) {
    this.subAccountAddress = subAccountAddress;
  }


  public BalanceExemption currency(Currency currency) {
    this.currency = currency;
    return this;
  }

   /**
   * Get currency
   * @return currency
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "")
  @JsonProperty(JSON_PROPERTY_CURRENCY)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public Currency getCurrency() {
    return currency;
  }


  @JsonProperty(JSON_PROPERTY_CURRENCY)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setCurrency(Currency currency) {
    this.currency = currency;
  }


  public BalanceExemption exemptionType(ExemptionType exemptionType) {
    this.exemptionType = exemptionType;
    return this;
  }

   /**
   * Get exemptionType
   * @return exemptionType
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "")
  @JsonProperty(JSON_PROPERTY_EXEMPTION_TYPE)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public ExemptionType getExemptionType() {
    return exemptionType;
  }


  @JsonProperty(JSON_PROPERTY_EXEMPTION_TYPE)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setExemptionType(ExemptionType exemptionType) {
    this.exemptionType = exemptionType;
  }


  /**
   * Return true if this BalanceExemption object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    BalanceExemption balanceExemption = (BalanceExemption) o;
    return Objects.equals(this.subAccountAddress, balanceExemption.subAccountAddress) &&
        Objects.equals(this.currency, balanceExemption.currency) &&
        Objects.equals(this.exemptionType, balanceExemption.exemptionType);
  }

  @Override
  public int hashCode() {
    return Objects.hash(subAccountAddress, currency, exemptionType);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class BalanceExemption {\n");
    sb.append("    subAccountAddress: ").append(toIndentedString(subAccountAddress)).append("\n");
    sb.append("    currency: ").append(toIndentedString(currency)).append("\n");
    sb.append("    exemptionType: ").append(toIndentedString(exemptionType)).append("\n");
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

