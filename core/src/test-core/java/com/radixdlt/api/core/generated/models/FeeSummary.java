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
 * FeeSummary
 */
@JsonPropertyOrder({
  FeeSummary.JSON_PROPERTY_EXECUTION_COST_UNITS_CONSUMED,
  FeeSummary.JSON_PROPERTY_FINALIZATION_COST_UNITS_CONSUMED,
  FeeSummary.JSON_PROPERTY_XRD_TOTAL_EXECUTION_COST,
  FeeSummary.JSON_PROPERTY_XRD_TOTAL_FINALIZATION_COST,
  FeeSummary.JSON_PROPERTY_XRD_TOTAL_ROYALTY_COST,
  FeeSummary.JSON_PROPERTY_XRD_TOTAL_STORAGE_COST,
  FeeSummary.JSON_PROPERTY_XRD_TOTAL_TIPPING_COST
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class FeeSummary {
  public static final String JSON_PROPERTY_EXECUTION_COST_UNITS_CONSUMED = "execution_cost_units_consumed";
  private Long executionCostUnitsConsumed;

  public static final String JSON_PROPERTY_FINALIZATION_COST_UNITS_CONSUMED = "finalization_cost_units_consumed";
  private Long finalizationCostUnitsConsumed;

  public static final String JSON_PROPERTY_XRD_TOTAL_EXECUTION_COST = "xrd_total_execution_cost";
  private String xrdTotalExecutionCost;

  public static final String JSON_PROPERTY_XRD_TOTAL_FINALIZATION_COST = "xrd_total_finalization_cost";
  private String xrdTotalFinalizationCost;

  public static final String JSON_PROPERTY_XRD_TOTAL_ROYALTY_COST = "xrd_total_royalty_cost";
  private String xrdTotalRoyaltyCost;

  public static final String JSON_PROPERTY_XRD_TOTAL_STORAGE_COST = "xrd_total_storage_cost";
  private String xrdTotalStorageCost;

  public static final String JSON_PROPERTY_XRD_TOTAL_TIPPING_COST = "xrd_total_tipping_cost";
  private String xrdTotalTippingCost;

  public FeeSummary() { 
  }

  public FeeSummary executionCostUnitsConsumed(Long executionCostUnitsConsumed) {
    this.executionCostUnitsConsumed = executionCostUnitsConsumed;
    return this;
  }

   /**
   * An integer between &#x60;0&#x60; and &#x60;2^32 - 1&#x60;, representing the amount of cost units consumed by the transaction execution.
   * minimum: 0
   * maximum: 4294967295
   * @return executionCostUnitsConsumed
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "An integer between `0` and `2^32 - 1`, representing the amount of cost units consumed by the transaction execution.")
  @JsonProperty(JSON_PROPERTY_EXECUTION_COST_UNITS_CONSUMED)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Long getExecutionCostUnitsConsumed() {
    return executionCostUnitsConsumed;
  }


  @JsonProperty(JSON_PROPERTY_EXECUTION_COST_UNITS_CONSUMED)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setExecutionCostUnitsConsumed(Long executionCostUnitsConsumed) {
    this.executionCostUnitsConsumed = executionCostUnitsConsumed;
  }


  public FeeSummary finalizationCostUnitsConsumed(Long finalizationCostUnitsConsumed) {
    this.finalizationCostUnitsConsumed = finalizationCostUnitsConsumed;
    return this;
  }

   /**
   * An integer between &#x60;0&#x60; and &#x60;2^32 - 1&#x60;, representing the amount of cost units consumed by the transaction finalization.
   * minimum: 0
   * maximum: 4294967295
   * @return finalizationCostUnitsConsumed
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "An integer between `0` and `2^32 - 1`, representing the amount of cost units consumed by the transaction finalization.")
  @JsonProperty(JSON_PROPERTY_FINALIZATION_COST_UNITS_CONSUMED)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Long getFinalizationCostUnitsConsumed() {
    return finalizationCostUnitsConsumed;
  }


  @JsonProperty(JSON_PROPERTY_FINALIZATION_COST_UNITS_CONSUMED)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setFinalizationCostUnitsConsumed(Long finalizationCostUnitsConsumed) {
    this.finalizationCostUnitsConsumed = finalizationCostUnitsConsumed;
  }


  public FeeSummary xrdTotalExecutionCost(String xrdTotalExecutionCost) {
    this.xrdTotalExecutionCost = xrdTotalExecutionCost;
    return this;
  }

   /**
   * The string-encoded decimal representing the total amount of XRD burned in the transaction as part of execution costs. A decimal is formed of some signed integer &#x60;m&#x60; of attos (&#x60;10^(-18)&#x60;) units, where &#x60;-2^(192 - 1) &lt;&#x3D; m &lt; 2^(192 - 1)&#x60;. 
   * @return xrdTotalExecutionCost
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The string-encoded decimal representing the total amount of XRD burned in the transaction as part of execution costs. A decimal is formed of some signed integer `m` of attos (`10^(-18)`) units, where `-2^(192 - 1) <= m < 2^(192 - 1)`. ")
  @JsonProperty(JSON_PROPERTY_XRD_TOTAL_EXECUTION_COST)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getXrdTotalExecutionCost() {
    return xrdTotalExecutionCost;
  }


  @JsonProperty(JSON_PROPERTY_XRD_TOTAL_EXECUTION_COST)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setXrdTotalExecutionCost(String xrdTotalExecutionCost) {
    this.xrdTotalExecutionCost = xrdTotalExecutionCost;
  }


  public FeeSummary xrdTotalFinalizationCost(String xrdTotalFinalizationCost) {
    this.xrdTotalFinalizationCost = xrdTotalFinalizationCost;
    return this;
  }

   /**
   * The string-encoded decimal representing the total amount of XRD burned in the transaction as part of finalization costs. A decimal is formed of some signed integer &#x60;m&#x60; of attos (&#x60;10^(-18)&#x60;) units, where &#x60;-2^(192 - 1) &lt;&#x3D; m &lt; 2^(192 - 1)&#x60;. 
   * @return xrdTotalFinalizationCost
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The string-encoded decimal representing the total amount of XRD burned in the transaction as part of finalization costs. A decimal is formed of some signed integer `m` of attos (`10^(-18)`) units, where `-2^(192 - 1) <= m < 2^(192 - 1)`. ")
  @JsonProperty(JSON_PROPERTY_XRD_TOTAL_FINALIZATION_COST)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getXrdTotalFinalizationCost() {
    return xrdTotalFinalizationCost;
  }


  @JsonProperty(JSON_PROPERTY_XRD_TOTAL_FINALIZATION_COST)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setXrdTotalFinalizationCost(String xrdTotalFinalizationCost) {
    this.xrdTotalFinalizationCost = xrdTotalFinalizationCost;
  }


  public FeeSummary xrdTotalRoyaltyCost(String xrdTotalRoyaltyCost) {
    this.xrdTotalRoyaltyCost = xrdTotalRoyaltyCost;
    return this;
  }

   /**
   * The string-encoded decimal representing the total amount of XRD paid in royalties as part of the transaction. A decimal is formed of some signed integer &#x60;m&#x60; of attos (&#x60;10^(-18)&#x60;) units, where &#x60;-2^(192 - 1) &lt;&#x3D; m &lt; 2^(192 - 1)&#x60;. 
   * @return xrdTotalRoyaltyCost
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The string-encoded decimal representing the total amount of XRD paid in royalties as part of the transaction. A decimal is formed of some signed integer `m` of attos (`10^(-18)`) units, where `-2^(192 - 1) <= m < 2^(192 - 1)`. ")
  @JsonProperty(JSON_PROPERTY_XRD_TOTAL_ROYALTY_COST)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getXrdTotalRoyaltyCost() {
    return xrdTotalRoyaltyCost;
  }


  @JsonProperty(JSON_PROPERTY_XRD_TOTAL_ROYALTY_COST)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setXrdTotalRoyaltyCost(String xrdTotalRoyaltyCost) {
    this.xrdTotalRoyaltyCost = xrdTotalRoyaltyCost;
  }


  public FeeSummary xrdTotalStorageCost(String xrdTotalStorageCost) {
    this.xrdTotalStorageCost = xrdTotalStorageCost;
    return this;
  }

   /**
   * The string-encoded decimal representing the total amount of XRD paid in state expansion costs as part of the transaction. A decimal is formed of some signed integer &#x60;m&#x60; of attos (&#x60;10^(-18)&#x60;) units, where &#x60;-2^(192 - 1) &lt;&#x3D; m &lt; 2^(192 - 1)&#x60;. 
   * @return xrdTotalStorageCost
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The string-encoded decimal representing the total amount of XRD paid in state expansion costs as part of the transaction. A decimal is formed of some signed integer `m` of attos (`10^(-18)`) units, where `-2^(192 - 1) <= m < 2^(192 - 1)`. ")
  @JsonProperty(JSON_PROPERTY_XRD_TOTAL_STORAGE_COST)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getXrdTotalStorageCost() {
    return xrdTotalStorageCost;
  }


  @JsonProperty(JSON_PROPERTY_XRD_TOTAL_STORAGE_COST)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setXrdTotalStorageCost(String xrdTotalStorageCost) {
    this.xrdTotalStorageCost = xrdTotalStorageCost;
  }


  public FeeSummary xrdTotalTippingCost(String xrdTotalTippingCost) {
    this.xrdTotalTippingCost = xrdTotalTippingCost;
    return this;
  }

   /**
   * The string-encoded decimal representing the total amount of XRD tipped to validators in the transaction. A decimal is formed of some signed integer &#x60;m&#x60; of attos (&#x60;10^(-18)&#x60;) units, where &#x60;-2^(192 - 1) &lt;&#x3D; m &lt; 2^(192 - 1)&#x60;. 
   * @return xrdTotalTippingCost
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The string-encoded decimal representing the total amount of XRD tipped to validators in the transaction. A decimal is formed of some signed integer `m` of attos (`10^(-18)`) units, where `-2^(192 - 1) <= m < 2^(192 - 1)`. ")
  @JsonProperty(JSON_PROPERTY_XRD_TOTAL_TIPPING_COST)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getXrdTotalTippingCost() {
    return xrdTotalTippingCost;
  }


  @JsonProperty(JSON_PROPERTY_XRD_TOTAL_TIPPING_COST)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setXrdTotalTippingCost(String xrdTotalTippingCost) {
    this.xrdTotalTippingCost = xrdTotalTippingCost;
  }


  /**
   * Return true if this FeeSummary object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    FeeSummary feeSummary = (FeeSummary) o;
    return Objects.equals(this.executionCostUnitsConsumed, feeSummary.executionCostUnitsConsumed) &&
        Objects.equals(this.finalizationCostUnitsConsumed, feeSummary.finalizationCostUnitsConsumed) &&
        Objects.equals(this.xrdTotalExecutionCost, feeSummary.xrdTotalExecutionCost) &&
        Objects.equals(this.xrdTotalFinalizationCost, feeSummary.xrdTotalFinalizationCost) &&
        Objects.equals(this.xrdTotalRoyaltyCost, feeSummary.xrdTotalRoyaltyCost) &&
        Objects.equals(this.xrdTotalStorageCost, feeSummary.xrdTotalStorageCost) &&
        Objects.equals(this.xrdTotalTippingCost, feeSummary.xrdTotalTippingCost);
  }

  @Override
  public int hashCode() {
    return Objects.hash(executionCostUnitsConsumed, finalizationCostUnitsConsumed, xrdTotalExecutionCost, xrdTotalFinalizationCost, xrdTotalRoyaltyCost, xrdTotalStorageCost, xrdTotalTippingCost);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class FeeSummary {\n");
    sb.append("    executionCostUnitsConsumed: ").append(toIndentedString(executionCostUnitsConsumed)).append("\n");
    sb.append("    finalizationCostUnitsConsumed: ").append(toIndentedString(finalizationCostUnitsConsumed)).append("\n");
    sb.append("    xrdTotalExecutionCost: ").append(toIndentedString(xrdTotalExecutionCost)).append("\n");
    sb.append("    xrdTotalFinalizationCost: ").append(toIndentedString(xrdTotalFinalizationCost)).append("\n");
    sb.append("    xrdTotalRoyaltyCost: ").append(toIndentedString(xrdTotalRoyaltyCost)).append("\n");
    sb.append("    xrdTotalStorageCost: ").append(toIndentedString(xrdTotalStorageCost)).append("\n");
    sb.append("    xrdTotalTippingCost: ").append(toIndentedString(xrdTotalTippingCost)).append("\n");
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

