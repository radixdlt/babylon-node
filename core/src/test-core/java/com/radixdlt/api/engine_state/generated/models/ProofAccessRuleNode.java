/*
 * Engine State API (Beta)
 * **This API is currently in Beta**  This specification may experience breaking changes as part of Babylon Node releases. Such changes will be clearly mentioned in the [babylon-node release notes](https://github.com/radixdlt/babylon-node/releases). We advise against using this API for business-critical integrations before the `version` indicated above becomes stable, which is expected in Q4 of 2024.  This API provides a complete view of the current ledger state, operating at a relatively low level (i.e. returning Entities' data and type information in a generic way, without interpreting specifics of different native or custom components).  It mirrors how the Radix Engine views the ledger state in its \"System\" layer, and thus can be useful for Scrypto developers, who need to inspect how the Engine models and stores their application's state, or how an interface / authentication scheme of another component looks like. 
 *
 * The version of the OpenAPI document: v1.2.2
 * 
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */


package com.radixdlt.api.engine_state.generated.models;

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
import com.radixdlt.api.engine_state.generated.models.AccessRuleNode;
import com.radixdlt.api.engine_state.generated.models.AccessRuleNodeType;
import com.radixdlt.api.engine_state.generated.models.AllOfAccessRuleNode;
import com.radixdlt.api.engine_state.generated.models.AnyOfAccessRuleNode;
import com.radixdlt.api.engine_state.generated.models.ProofAccessRuleNode;
import com.radixdlt.api.engine_state.generated.models.ProofAccessRuleNodeAllOf;
import com.radixdlt.api.engine_state.generated.models.ProofRule;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


import com.radixdlt.api.engine_state.generated.client.JSON;
/**
 * ProofAccessRuleNode
 */
@JsonPropertyOrder({
  ProofAccessRuleNode.JSON_PROPERTY_PROOF_RULE
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
@JsonIgnoreProperties(
  value = "type", // ignore manually set type, it will be automatically generated by Jackson during serialization
  allowSetters = true // allows the type to be set during deserialization
)
@JsonTypeInfo(use = JsonTypeInfo.Id.NAME, include = JsonTypeInfo.As.PROPERTY, property = "type", visible = true)
@JsonSubTypes({
  @JsonSubTypes.Type(value = AllOfAccessRuleNode.class, name = "AllOf"),
  @JsonSubTypes.Type(value = AnyOfAccessRuleNode.class, name = "AnyOf"),
  @JsonSubTypes.Type(value = ProofAccessRuleNode.class, name = "ProofRule"),
})

public class ProofAccessRuleNode extends AccessRuleNode {
  public static final String JSON_PROPERTY_PROOF_RULE = "proof_rule";
  private ProofRule proofRule;

  public ProofAccessRuleNode() { 
  }

  public ProofAccessRuleNode proofRule(ProofRule proofRule) {
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

  public ProofRule getProofRule() {
    return proofRule;
  }


  @JsonProperty(JSON_PROPERTY_PROOF_RULE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setProofRule(ProofRule proofRule) {
    this.proofRule = proofRule;
  }


  /**
   * Return true if this ProofAccessRuleNode object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    ProofAccessRuleNode proofAccessRuleNode = (ProofAccessRuleNode) o;
    return Objects.equals(this.proofRule, proofAccessRuleNode.proofRule) &&
        super.equals(o);
  }

  @Override
  public int hashCode() {
    return Objects.hash(proofRule, super.hashCode());
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class ProofAccessRuleNode {\n");
    sb.append("    ").append(toIndentedString(super.toString())).append("\n");
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

static {
  // Initialize and register the discriminator mappings.
  Map<String, Class<?>> mappings = new HashMap<String, Class<?>>();
  mappings.put("AllOf", AllOfAccessRuleNode.class);
  mappings.put("AnyOf", AnyOfAccessRuleNode.class);
  mappings.put("ProofRule", ProofAccessRuleNode.class);
  mappings.put("ProofAccessRuleNode", ProofAccessRuleNode.class);
  JSON.registerDiscriminator(ProofAccessRuleNode.class, "type", mappings);
}
}

