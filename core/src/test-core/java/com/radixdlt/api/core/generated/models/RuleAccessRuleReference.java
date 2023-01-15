/*
 * Babylon Core API
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node. It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Heavy load may impact the node's function.  If you require queries against historical ledger state, you may also wish to consider using the [Gateway API](https://betanet-gateway.redoc.ly/). 
 *
 * The version of the OpenAPI document: 0.2.0
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
import com.radixdlt.api.core.generated.models.AccessRule;
import com.radixdlt.api.core.generated.models.AccessRuleReferenceBase;
import com.radixdlt.api.core.generated.models.AccessRuleReferenceType;
import com.radixdlt.api.core.generated.models.RuleAccessRuleReferenceAllOf;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * RuleAccessRuleReference
 */
@JsonPropertyOrder({
  RuleAccessRuleReference.JSON_PROPERTY_TYPE,
  RuleAccessRuleReference.JSON_PROPERTY_ACCESS_RULE
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class RuleAccessRuleReference {
  public static final String JSON_PROPERTY_TYPE = "type";
  private AccessRuleReferenceType type;

  public static final String JSON_PROPERTY_ACCESS_RULE = "access_rule";
  private AccessRule accessRule;

  public RuleAccessRuleReference() { 
  }

  public RuleAccessRuleReference type(AccessRuleReferenceType type) {
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

  public AccessRuleReferenceType getType() {
    return type;
  }


  @JsonProperty(JSON_PROPERTY_TYPE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setType(AccessRuleReferenceType type) {
    this.type = type;
  }


  public RuleAccessRuleReference accessRule(AccessRule accessRule) {
    this.accessRule = accessRule;
    return this;
  }

   /**
   * Get accessRule
   * @return accessRule
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_ACCESS_RULE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public AccessRule getAccessRule() {
    return accessRule;
  }


  @JsonProperty(JSON_PROPERTY_ACCESS_RULE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setAccessRule(AccessRule accessRule) {
    this.accessRule = accessRule;
  }


  /**
   * Return true if this RuleAccessRuleReference object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    RuleAccessRuleReference ruleAccessRuleReference = (RuleAccessRuleReference) o;
    return Objects.equals(this.type, ruleAccessRuleReference.type) &&
        Objects.equals(this.accessRule, ruleAccessRuleReference.accessRule);
  }

  @Override
  public int hashCode() {
    return Objects.hash(type, accessRule);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class RuleAccessRuleReference {\n");
    sb.append("    type: ").append(toIndentedString(type)).append("\n");
    sb.append("    accessRule: ").append(toIndentedString(accessRule)).append("\n");
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

