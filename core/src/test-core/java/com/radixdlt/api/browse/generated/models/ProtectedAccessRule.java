/*
 * Browse API
 * This API provides a complete view of the current ledger state, operating at a relatively low level (i.e. returning Entities' data and type information in a generic way, without interpreting specifics of different native or custom components).  It mirrors how the Radix Engine views the ledger state in its \"System\" layer, and thus can be useful for Scrypto developers, who need to inspect how the Engine models and stores their application's state, or how an interface / authentication scheme of another component looks like. 
 *
 * The version of the OpenAPI document: v0.0.1
 * 
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */


package com.radixdlt.api.browse.generated.models;

import java.util.Objects;
import java.util.Arrays;
import java.util.Map;
import java.util.HashMap;
import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonSubTypes;
import com.fasterxml.jackson.annotation.JsonTypeInfo;
import com.fasterxml.jackson.annotation.JsonTypeName;
import com.fasterxml.jackson.annotation.JsonValue;
import com.radixdlt.api.browse.generated.models.AccessRule;
import com.radixdlt.api.browse.generated.models.AccessRuleNode;
import com.radixdlt.api.browse.generated.models.AccessRuleType;
import com.radixdlt.api.browse.generated.models.AllowAllAccessRule;
import com.radixdlt.api.browse.generated.models.DenyAllAccessRule;
import com.radixdlt.api.browse.generated.models.ProtectedAccessRule;
import com.radixdlt.api.browse.generated.models.ProtectedAccessRuleAllOf;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


import com.radixdlt.api.browse.generated.client.JSON;
/**
 * ProtectedAccessRule
 */
@JsonPropertyOrder({
  ProtectedAccessRule.JSON_PROPERTY_ACCESS_RULE
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
@JsonIgnoreProperties(
  value = "type", // ignore manually set type, it will be automatically generated by Jackson during serialization
  allowSetters = true // allows the type to be set during deserialization
)
@JsonTypeInfo(use = JsonTypeInfo.Id.NAME, include = JsonTypeInfo.As.PROPERTY, property = "type", visible = true)
@JsonSubTypes({
  @JsonSubTypes.Type(value = AllowAllAccessRule.class, name = "AllowAll"),
  @JsonSubTypes.Type(value = DenyAllAccessRule.class, name = "DenyAll"),
  @JsonSubTypes.Type(value = ProtectedAccessRule.class, name = "Protected"),
})

public class ProtectedAccessRule extends AccessRule {
  public static final String JSON_PROPERTY_ACCESS_RULE = "access_rule";
  private AccessRuleNode accessRule;

  public ProtectedAccessRule() { 
  }

  public ProtectedAccessRule accessRule(AccessRuleNode accessRule) {
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

  public AccessRuleNode getAccessRule() {
    return accessRule;
  }


  @JsonProperty(JSON_PROPERTY_ACCESS_RULE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setAccessRule(AccessRuleNode accessRule) {
    this.accessRule = accessRule;
  }


  /**
   * Return true if this ProtectedAccessRule object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    ProtectedAccessRule protectedAccessRule = (ProtectedAccessRule) o;
    return Objects.equals(this.accessRule, protectedAccessRule.accessRule) &&
        super.equals(o);
  }

  @Override
  public int hashCode() {
    return Objects.hash(accessRule, super.hashCode());
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class ProtectedAccessRule {\n");
    sb.append("    ").append(toIndentedString(super.toString())).append("\n");
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

static {
  // Initialize and register the discriminator mappings.
  Map<String, Class<?>> mappings = new HashMap<String, Class<?>>();
  mappings.put("AllowAll", AllowAllAccessRule.class);
  mappings.put("DenyAll", DenyAllAccessRule.class);
  mappings.put("Protected", ProtectedAccessRule.class);
  mappings.put("ProtectedAccessRule", ProtectedAccessRule.class);
  JSON.registerDiscriminator(ProtectedAccessRule.class, "type", mappings);
}
}

