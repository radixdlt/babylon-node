/*
 * Radix Core API
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.3.0
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
import com.radixdlt.api.core.generated.models.MethodAccessibility;
import com.radixdlt.api.core.generated.models.RoleDetails;
import com.radixdlt.api.core.generated.models.RoleSpecification;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * StaticRoleDefinitionAuthTemplate
 */
@JsonPropertyOrder({
  StaticRoleDefinitionAuthTemplate.JSON_PROPERTY_ROLE_SPECIFICATION,
  StaticRoleDefinitionAuthTemplate.JSON_PROPERTY_ROLES,
  StaticRoleDefinitionAuthTemplate.JSON_PROPERTY_METHOD_ACCESSIBILITY_MAP
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class StaticRoleDefinitionAuthTemplate {
  public static final String JSON_PROPERTY_ROLE_SPECIFICATION = "role_specification";
  private RoleSpecification roleSpecification;

  public static final String JSON_PROPERTY_ROLES = "roles";
  private Map<String, RoleDetails> roles = null;

  public static final String JSON_PROPERTY_METHOD_ACCESSIBILITY_MAP = "method_accessibility_map";
  private Map<String, MethodAccessibility> methodAccessibilityMap = new HashMap<>();

  public StaticRoleDefinitionAuthTemplate() { 
  }

  public StaticRoleDefinitionAuthTemplate roleSpecification(RoleSpecification roleSpecification) {
    this.roleSpecification = roleSpecification;
    return this;
  }

   /**
   * Get roleSpecification
   * @return roleSpecification
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_ROLE_SPECIFICATION)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public RoleSpecification getRoleSpecification() {
    return roleSpecification;
  }


  @JsonProperty(JSON_PROPERTY_ROLE_SPECIFICATION)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setRoleSpecification(RoleSpecification roleSpecification) {
    this.roleSpecification = roleSpecification;
  }


  public StaticRoleDefinitionAuthTemplate roles(Map<String, RoleDetails> roles) {
    this.roles = roles;
    return this;
  }

  public StaticRoleDefinitionAuthTemplate putRolesItem(String key, RoleDetails rolesItem) {
    if (this.roles == null) {
      this.roles = new HashMap<>();
    }
    this.roles.put(key, rolesItem);
    return this;
  }

   /**
   * A map from role name to role details
   * @return roles
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "A map from role name to role details")
  @JsonProperty(JSON_PROPERTY_ROLES)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public Map<String, RoleDetails> getRoles() {
    return roles;
  }


  @JsonProperty(JSON_PROPERTY_ROLES)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setRoles(Map<String, RoleDetails> roles) {
    this.roles = roles;
  }


  public StaticRoleDefinitionAuthTemplate methodAccessibilityMap(Map<String, MethodAccessibility> methodAccessibilityMap) {
    this.methodAccessibilityMap = methodAccessibilityMap;
    return this;
  }

  public StaticRoleDefinitionAuthTemplate putMethodAccessibilityMapItem(String key, MethodAccessibility methodAccessibilityMapItem) {
    this.methodAccessibilityMap.put(key, methodAccessibilityMapItem);
    return this;
  }

   /**
   * A map from a method identifier to MethodAccessibility
   * @return methodAccessibilityMap
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "A map from a method identifier to MethodAccessibility")
  @JsonProperty(JSON_PROPERTY_METHOD_ACCESSIBILITY_MAP)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Map<String, MethodAccessibility> getMethodAccessibilityMap() {
    return methodAccessibilityMap;
  }


  @JsonProperty(JSON_PROPERTY_METHOD_ACCESSIBILITY_MAP)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setMethodAccessibilityMap(Map<String, MethodAccessibility> methodAccessibilityMap) {
    this.methodAccessibilityMap = methodAccessibilityMap;
  }


  /**
   * Return true if this StaticRoleDefinitionAuthTemplate object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    StaticRoleDefinitionAuthTemplate staticRoleDefinitionAuthTemplate = (StaticRoleDefinitionAuthTemplate) o;
    return Objects.equals(this.roleSpecification, staticRoleDefinitionAuthTemplate.roleSpecification) &&
        Objects.equals(this.roles, staticRoleDefinitionAuthTemplate.roles) &&
        Objects.equals(this.methodAccessibilityMap, staticRoleDefinitionAuthTemplate.methodAccessibilityMap);
  }

  @Override
  public int hashCode() {
    return Objects.hash(roleSpecification, roles, methodAccessibilityMap);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class StaticRoleDefinitionAuthTemplate {\n");
    sb.append("    roleSpecification: ").append(toIndentedString(roleSpecification)).append("\n");
    sb.append("    roles: ").append(toIndentedString(roles)).append("\n");
    sb.append("    methodAccessibilityMap: ").append(toIndentedString(methodAccessibilityMap)).append("\n");
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

