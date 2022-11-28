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
import com.radixdlt.api.core.generated.models.DynamicProofRule;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * ProofDynamicAuthRuleAllOf
 */
@JsonPropertyOrder({
  ProofDynamicAuthRuleAllOf.JSON_PROPERTY_PROOF_RULE
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class ProofDynamicAuthRuleAllOf {
  public static final String JSON_PROPERTY_PROOF_RULE = "proof_rule";
  private DynamicProofRule proofRule;

  public ProofDynamicAuthRuleAllOf() { 
  }

  public ProofDynamicAuthRuleAllOf proofRule(DynamicProofRule proofRule) {
    this.proofRule = proofRule;
    return this;
  }

   /**
   * Get proofRule
   * @return proofRule
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_PROOF_RULE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public DynamicProofRule getProofRule() {
    return proofRule;
  }


  @JsonProperty(JSON_PROPERTY_PROOF_RULE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setProofRule(DynamicProofRule proofRule) {
    this.proofRule = proofRule;
  }


  /**
   * Return true if this ProofDynamicAuthRule_allOf object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    ProofDynamicAuthRuleAllOf proofDynamicAuthRuleAllOf = (ProofDynamicAuthRuleAllOf) o;
    return Objects.equals(this.proofRule, proofDynamicAuthRuleAllOf.proofRule);
  }

  @Override
  public int hashCode() {
    return Objects.hash(proofRule);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class ProofDynamicAuthRuleAllOf {\n");
    sb.append("    proofRule: ").append(toIndentedString(proofRule)).append("\n");
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

