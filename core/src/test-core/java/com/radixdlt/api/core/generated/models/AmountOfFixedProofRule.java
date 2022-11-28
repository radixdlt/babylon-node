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
import com.radixdlt.api.core.generated.models.AmountOfFixedProofRuleAllOf;
import com.radixdlt.api.core.generated.models.FixedProofRuleBase;
import com.radixdlt.api.core.generated.models.FixedProofRuleType;
import com.radixdlt.api.core.generated.models.FixedResourceDescriptor;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * AmountOfFixedProofRule
 */
@JsonPropertyOrder({
  AmountOfFixedProofRule.JSON_PROPERTY_TYPE,
  AmountOfFixedProofRule.JSON_PROPERTY_AMOUNT,
  AmountOfFixedProofRule.JSON_PROPERTY_RESOURCE
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class AmountOfFixedProofRule {
  public static final String JSON_PROPERTY_TYPE = "type";
  private FixedProofRuleType type;

  public static final String JSON_PROPERTY_AMOUNT = "amount";
  private String amount;

  public static final String JSON_PROPERTY_RESOURCE = "resource";
  private FixedResourceDescriptor resource;

  public AmountOfFixedProofRule() { 
  }

  public AmountOfFixedProofRule type(FixedProofRuleType type) {
    this.type = type;
    return this;
  }

   /**
   * Get type
   * @return type
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_TYPE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public FixedProofRuleType getType() {
    return type;
  }


  @JsonProperty(JSON_PROPERTY_TYPE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setType(FixedProofRuleType type) {
    this.type = type;
  }


  public AmountOfFixedProofRule amount(String amount) {
    this.amount = amount;
    return this;
  }

   /**
   * The string-encoded decimal representing the amount of resource required to pass the proof rule. A decimal is formed of some signed integer &#x60;m&#x60; of attos (&#x60;10^(-18)&#x60;) units, where &#x60;-2^(256 - 1) &lt;&#x3D; m &lt; 2^(256 - 1)&#x60;. 
   * @return amount
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The string-encoded decimal representing the amount of resource required to pass the proof rule. A decimal is formed of some signed integer `m` of attos (`10^(-18)`) units, where `-2^(256 - 1) <= m < 2^(256 - 1)`. ")
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


  public AmountOfFixedProofRule resource(FixedResourceDescriptor resource) {
    this.resource = resource;
    return this;
  }

   /**
   * Get resource
   * @return resource
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_RESOURCE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public FixedResourceDescriptor getResource() {
    return resource;
  }


  @JsonProperty(JSON_PROPERTY_RESOURCE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setResource(FixedResourceDescriptor resource) {
    this.resource = resource;
  }


  /**
   * Return true if this AmountOfFixedProofRule object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    AmountOfFixedProofRule amountOfFixedProofRule = (AmountOfFixedProofRule) o;
    return Objects.equals(this.type, amountOfFixedProofRule.type) &&
        Objects.equals(this.amount, amountOfFixedProofRule.amount) &&
        Objects.equals(this.resource, amountOfFixedProofRule.resource);
  }

  @Override
  public int hashCode() {
    return Objects.hash(type, amount, resource);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class AmountOfFixedProofRule {\n");
    sb.append("    type: ").append(toIndentedString(type)).append("\n");
    sb.append("    amount: ").append(toIndentedString(amount)).append("\n");
    sb.append("    resource: ").append(toIndentedString(resource)).append("\n");
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

