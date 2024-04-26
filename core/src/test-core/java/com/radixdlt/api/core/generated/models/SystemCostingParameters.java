/*
 * Radix Core API - Babylon (Bottlenose)
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.2.0
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
 * SystemCostingParameters
 */
@JsonPropertyOrder({
  SystemCostingParameters.JSON_PROPERTY_EXECUTION_COST_UNIT_PRICE,
  SystemCostingParameters.JSON_PROPERTY_EXECUTION_COST_UNIT_LIMIT,
  SystemCostingParameters.JSON_PROPERTY_EXECUTION_COST_UNIT_LOAN,
  SystemCostingParameters.JSON_PROPERTY_FINALIZATION_COST_UNIT_PRICE,
  SystemCostingParameters.JSON_PROPERTY_FINALIZATION_COST_UNIT_LIMIT,
  SystemCostingParameters.JSON_PROPERTY_XRD_USD_PRICE,
  SystemCostingParameters.JSON_PROPERTY_XRD_STORAGE_PRICE,
  SystemCostingParameters.JSON_PROPERTY_XRD_ARCHIVE_STORAGE_PRICE
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class SystemCostingParameters {
  public static final String JSON_PROPERTY_EXECUTION_COST_UNIT_PRICE = "execution_cost_unit_price";
  private String executionCostUnitPrice;

  public static final String JSON_PROPERTY_EXECUTION_COST_UNIT_LIMIT = "execution_cost_unit_limit";
  private Long executionCostUnitLimit;

  public static final String JSON_PROPERTY_EXECUTION_COST_UNIT_LOAN = "execution_cost_unit_loan";
  private Long executionCostUnitLoan;

  public static final String JSON_PROPERTY_FINALIZATION_COST_UNIT_PRICE = "finalization_cost_unit_price";
  private String finalizationCostUnitPrice;

  public static final String JSON_PROPERTY_FINALIZATION_COST_UNIT_LIMIT = "finalization_cost_unit_limit";
  private Long finalizationCostUnitLimit;

  public static final String JSON_PROPERTY_XRD_USD_PRICE = "xrd_usd_price";
  private String xrdUsdPrice;

  public static final String JSON_PROPERTY_XRD_STORAGE_PRICE = "xrd_storage_price";
  private String xrdStoragePrice;

  public static final String JSON_PROPERTY_XRD_ARCHIVE_STORAGE_PRICE = "xrd_archive_storage_price";
  private String xrdArchiveStoragePrice;

  public SystemCostingParameters() { 
  }

  public SystemCostingParameters executionCostUnitPrice(String executionCostUnitPrice) {
    this.executionCostUnitPrice = executionCostUnitPrice;
    return this;
  }

   /**
   * The string-encoded decimal representing the XRD price of a single cost unit of transaction execution. A decimal is formed of some signed integer &#x60;m&#x60; of attos (&#x60;10^(-18)&#x60;) units, where &#x60;-2^(192 - 1) &lt;&#x3D; m &lt; 2^(192 - 1)&#x60;. 
   * @return executionCostUnitPrice
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The string-encoded decimal representing the XRD price of a single cost unit of transaction execution. A decimal is formed of some signed integer `m` of attos (`10^(-18)`) units, where `-2^(192 - 1) <= m < 2^(192 - 1)`. ")
  @JsonProperty(JSON_PROPERTY_EXECUTION_COST_UNIT_PRICE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getExecutionCostUnitPrice() {
    return executionCostUnitPrice;
  }


  @JsonProperty(JSON_PROPERTY_EXECUTION_COST_UNIT_PRICE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setExecutionCostUnitPrice(String executionCostUnitPrice) {
    this.executionCostUnitPrice = executionCostUnitPrice;
  }


  public SystemCostingParameters executionCostUnitLimit(Long executionCostUnitLimit) {
    this.executionCostUnitLimit = executionCostUnitLimit;
    return this;
  }

   /**
   * An integer between &#x60;0&#x60; and &#x60;2^32 - 1&#x60;, representing the maximum amount of cost units available for the transaction execution.
   * minimum: 0
   * maximum: 4294967295
   * @return executionCostUnitLimit
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "An integer between `0` and `2^32 - 1`, representing the maximum amount of cost units available for the transaction execution.")
  @JsonProperty(JSON_PROPERTY_EXECUTION_COST_UNIT_LIMIT)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Long getExecutionCostUnitLimit() {
    return executionCostUnitLimit;
  }


  @JsonProperty(JSON_PROPERTY_EXECUTION_COST_UNIT_LIMIT)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setExecutionCostUnitLimit(Long executionCostUnitLimit) {
    this.executionCostUnitLimit = executionCostUnitLimit;
  }


  public SystemCostingParameters executionCostUnitLoan(Long executionCostUnitLoan) {
    this.executionCostUnitLoan = executionCostUnitLoan;
    return this;
  }

   /**
   * An integer between &#x60;0&#x60; and &#x60;2^32 - 1&#x60;, representing the maximum number of cost units which can be used before fee is locked from a vault.
   * minimum: 0
   * maximum: 4294967295
   * @return executionCostUnitLoan
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "An integer between `0` and `2^32 - 1`, representing the maximum number of cost units which can be used before fee is locked from a vault.")
  @JsonProperty(JSON_PROPERTY_EXECUTION_COST_UNIT_LOAN)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Long getExecutionCostUnitLoan() {
    return executionCostUnitLoan;
  }


  @JsonProperty(JSON_PROPERTY_EXECUTION_COST_UNIT_LOAN)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setExecutionCostUnitLoan(Long executionCostUnitLoan) {
    this.executionCostUnitLoan = executionCostUnitLoan;
  }


  public SystemCostingParameters finalizationCostUnitPrice(String finalizationCostUnitPrice) {
    this.finalizationCostUnitPrice = finalizationCostUnitPrice;
    return this;
  }

   /**
   * The string-encoded decimal representing the XRD price of a single cost unit of transaction finalization. A decimal is formed of some signed integer &#x60;m&#x60; of attos (&#x60;10^(-18)&#x60;) units, where &#x60;-2^(192 - 1) &lt;&#x3D; m &lt; 2^(192 - 1)&#x60;. 
   * @return finalizationCostUnitPrice
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The string-encoded decimal representing the XRD price of a single cost unit of transaction finalization. A decimal is formed of some signed integer `m` of attos (`10^(-18)`) units, where `-2^(192 - 1) <= m < 2^(192 - 1)`. ")
  @JsonProperty(JSON_PROPERTY_FINALIZATION_COST_UNIT_PRICE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getFinalizationCostUnitPrice() {
    return finalizationCostUnitPrice;
  }


  @JsonProperty(JSON_PROPERTY_FINALIZATION_COST_UNIT_PRICE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setFinalizationCostUnitPrice(String finalizationCostUnitPrice) {
    this.finalizationCostUnitPrice = finalizationCostUnitPrice;
  }


  public SystemCostingParameters finalizationCostUnitLimit(Long finalizationCostUnitLimit) {
    this.finalizationCostUnitLimit = finalizationCostUnitLimit;
    return this;
  }

   /**
   * An integer between &#x60;0&#x60; and &#x60;2^32 - 1&#x60;, representing the maximum amount of cost units available for the transaction finalization.
   * minimum: 0
   * maximum: 4294967295
   * @return finalizationCostUnitLimit
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "An integer between `0` and `2^32 - 1`, representing the maximum amount of cost units available for the transaction finalization.")
  @JsonProperty(JSON_PROPERTY_FINALIZATION_COST_UNIT_LIMIT)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Long getFinalizationCostUnitLimit() {
    return finalizationCostUnitLimit;
  }


  @JsonProperty(JSON_PROPERTY_FINALIZATION_COST_UNIT_LIMIT)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setFinalizationCostUnitLimit(Long finalizationCostUnitLimit) {
    this.finalizationCostUnitLimit = finalizationCostUnitLimit;
  }


  public SystemCostingParameters xrdUsdPrice(String xrdUsdPrice) {
    this.xrdUsdPrice = xrdUsdPrice;
    return this;
  }

   /**
   * The string-encoded decimal representing what amount of XRD is consumed by a Royalty of 1 USD. This is fixed for a given protocol version, so is not an accurate representation of the XRD price. A decimal is formed of some signed integer &#x60;m&#x60; of attos (&#x60;10^(-18)&#x60;) units, where &#x60;-2^(192 - 1) &lt;&#x3D; m &lt; 2^(192 - 1)&#x60;. 
   * @return xrdUsdPrice
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The string-encoded decimal representing what amount of XRD is consumed by a Royalty of 1 USD. This is fixed for a given protocol version, so is not an accurate representation of the XRD price. A decimal is formed of some signed integer `m` of attos (`10^(-18)`) units, where `-2^(192 - 1) <= m < 2^(192 - 1)`. ")
  @JsonProperty(JSON_PROPERTY_XRD_USD_PRICE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getXrdUsdPrice() {
    return xrdUsdPrice;
  }


  @JsonProperty(JSON_PROPERTY_XRD_USD_PRICE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setXrdUsdPrice(String xrdUsdPrice) {
    this.xrdUsdPrice = xrdUsdPrice;
  }


  public SystemCostingParameters xrdStoragePrice(String xrdStoragePrice) {
    this.xrdStoragePrice = xrdStoragePrice;
    return this;
  }

   /**
   * The string-encoded decimal representing the price of 1 byte of state storage, expressed in XRD. A decimal is formed of some signed integer &#x60;m&#x60; of attos (&#x60;10^(-18)&#x60;) units, where &#x60;-2^(192 - 1) &lt;&#x3D; m &lt; 2^(192 - 1)&#x60;. 
   * @return xrdStoragePrice
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The string-encoded decimal representing the price of 1 byte of state storage, expressed in XRD. A decimal is formed of some signed integer `m` of attos (`10^(-18)`) units, where `-2^(192 - 1) <= m < 2^(192 - 1)`. ")
  @JsonProperty(JSON_PROPERTY_XRD_STORAGE_PRICE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getXrdStoragePrice() {
    return xrdStoragePrice;
  }


  @JsonProperty(JSON_PROPERTY_XRD_STORAGE_PRICE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setXrdStoragePrice(String xrdStoragePrice) {
    this.xrdStoragePrice = xrdStoragePrice;
  }


  public SystemCostingParameters xrdArchiveStoragePrice(String xrdArchiveStoragePrice) {
    this.xrdArchiveStoragePrice = xrdArchiveStoragePrice;
    return this;
  }

   /**
   * The string-encoded decimal representing the price of 1 byte of archive storage, expressed in XRD. A decimal is formed of some signed integer &#x60;m&#x60; of attos (&#x60;10^(-18)&#x60;) units, where &#x60;-2^(192 - 1) &lt;&#x3D; m &lt; 2^(192 - 1)&#x60;. 
   * @return xrdArchiveStoragePrice
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The string-encoded decimal representing the price of 1 byte of archive storage, expressed in XRD. A decimal is formed of some signed integer `m` of attos (`10^(-18)`) units, where `-2^(192 - 1) <= m < 2^(192 - 1)`. ")
  @JsonProperty(JSON_PROPERTY_XRD_ARCHIVE_STORAGE_PRICE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getXrdArchiveStoragePrice() {
    return xrdArchiveStoragePrice;
  }


  @JsonProperty(JSON_PROPERTY_XRD_ARCHIVE_STORAGE_PRICE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setXrdArchiveStoragePrice(String xrdArchiveStoragePrice) {
    this.xrdArchiveStoragePrice = xrdArchiveStoragePrice;
  }


  /**
   * Return true if this SystemCostingParameters object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    SystemCostingParameters systemCostingParameters = (SystemCostingParameters) o;
    return Objects.equals(this.executionCostUnitPrice, systemCostingParameters.executionCostUnitPrice) &&
        Objects.equals(this.executionCostUnitLimit, systemCostingParameters.executionCostUnitLimit) &&
        Objects.equals(this.executionCostUnitLoan, systemCostingParameters.executionCostUnitLoan) &&
        Objects.equals(this.finalizationCostUnitPrice, systemCostingParameters.finalizationCostUnitPrice) &&
        Objects.equals(this.finalizationCostUnitLimit, systemCostingParameters.finalizationCostUnitLimit) &&
        Objects.equals(this.xrdUsdPrice, systemCostingParameters.xrdUsdPrice) &&
        Objects.equals(this.xrdStoragePrice, systemCostingParameters.xrdStoragePrice) &&
        Objects.equals(this.xrdArchiveStoragePrice, systemCostingParameters.xrdArchiveStoragePrice);
  }

  @Override
  public int hashCode() {
    return Objects.hash(executionCostUnitPrice, executionCostUnitLimit, executionCostUnitLoan, finalizationCostUnitPrice, finalizationCostUnitLimit, xrdUsdPrice, xrdStoragePrice, xrdArchiveStoragePrice);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class SystemCostingParameters {\n");
    sb.append("    executionCostUnitPrice: ").append(toIndentedString(executionCostUnitPrice)).append("\n");
    sb.append("    executionCostUnitLimit: ").append(toIndentedString(executionCostUnitLimit)).append("\n");
    sb.append("    executionCostUnitLoan: ").append(toIndentedString(executionCostUnitLoan)).append("\n");
    sb.append("    finalizationCostUnitPrice: ").append(toIndentedString(finalizationCostUnitPrice)).append("\n");
    sb.append("    finalizationCostUnitLimit: ").append(toIndentedString(finalizationCostUnitLimit)).append("\n");
    sb.append("    xrdUsdPrice: ").append(toIndentedString(xrdUsdPrice)).append("\n");
    sb.append("    xrdStoragePrice: ").append(toIndentedString(xrdStoragePrice)).append("\n");
    sb.append("    xrdArchiveStoragePrice: ").append(toIndentedString(xrdArchiveStoragePrice)).append("\n");
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

