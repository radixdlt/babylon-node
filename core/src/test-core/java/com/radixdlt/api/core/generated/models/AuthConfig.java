/*
 * Radix Core API
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.2.2
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
import com.radixdlt.api.core.generated.models.FunctionAuthType;
import com.radixdlt.api.core.generated.models.MethodAuthType;
import com.radixdlt.api.core.generated.models.StaticRoleDefinitionAuthTemplate;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * AuthConfig
 */
@JsonPropertyOrder({
  AuthConfig.JSON_PROPERTY_FUNCTION_AUTH_TYPE,
  AuthConfig.JSON_PROPERTY_FUNCTION_ACCESS_RULES,
  AuthConfig.JSON_PROPERTY_METHOD_AUTH_TYPE,
  AuthConfig.JSON_PROPERTY_METHOD_ROLES
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class AuthConfig {
  public static final String JSON_PROPERTY_FUNCTION_AUTH_TYPE = "function_auth_type";
  private FunctionAuthType functionAuthType;

  public static final String JSON_PROPERTY_FUNCTION_ACCESS_RULES = "function_access_rules";
  private Map<String, AccessRule> functionAccessRules = null;

  public static final String JSON_PROPERTY_METHOD_AUTH_TYPE = "method_auth_type";
  private MethodAuthType methodAuthType;

  public static final String JSON_PROPERTY_METHOD_ROLES = "method_roles";
  private StaticRoleDefinitionAuthTemplate methodRoles;

  public AuthConfig() { 
  }

  public AuthConfig functionAuthType(FunctionAuthType functionAuthType) {
    this.functionAuthType = functionAuthType;
    return this;
  }

   /**
   * Get functionAuthType
   * @return functionAuthType
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_FUNCTION_AUTH_TYPE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public FunctionAuthType getFunctionAuthType() {
    return functionAuthType;
  }


  @JsonProperty(JSON_PROPERTY_FUNCTION_AUTH_TYPE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setFunctionAuthType(FunctionAuthType functionAuthType) {
    this.functionAuthType = functionAuthType;
  }


  public AuthConfig functionAccessRules(Map<String, AccessRule> functionAccessRules) {
    this.functionAccessRules = functionAccessRules;
    return this;
  }

  public AuthConfig putFunctionAccessRulesItem(String key, AccessRule functionAccessRulesItem) {
    if (this.functionAccessRules == null) {
      this.functionAccessRules = new HashMap<>();
    }
    this.functionAccessRules.put(key, functionAccessRulesItem);
    return this;
  }

   /**
   * A map from a function name to AccessRule. Only exists if &#x60;function_auth_type&#x60; is set to &#x60;FunctionAccessRules&#x60;. 
   * @return functionAccessRules
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "A map from a function name to AccessRule. Only exists if `function_auth_type` is set to `FunctionAccessRules`. ")
  @JsonProperty(JSON_PROPERTY_FUNCTION_ACCESS_RULES)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public Map<String, AccessRule> getFunctionAccessRules() {
    return functionAccessRules;
  }


  @JsonProperty(JSON_PROPERTY_FUNCTION_ACCESS_RULES)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setFunctionAccessRules(Map<String, AccessRule> functionAccessRules) {
    this.functionAccessRules = functionAccessRules;
  }


  public AuthConfig methodAuthType(MethodAuthType methodAuthType) {
    this.methodAuthType = methodAuthType;
    return this;
  }

   /**
   * Get methodAuthType
   * @return methodAuthType
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_METHOD_AUTH_TYPE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public MethodAuthType getMethodAuthType() {
    return methodAuthType;
  }


  @JsonProperty(JSON_PROPERTY_METHOD_AUTH_TYPE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setMethodAuthType(MethodAuthType methodAuthType) {
    this.methodAuthType = methodAuthType;
  }


  public AuthConfig methodRoles(StaticRoleDefinitionAuthTemplate methodRoles) {
    this.methodRoles = methodRoles;
    return this;
  }

   /**
   * Get methodRoles
   * @return methodRoles
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "")
  @JsonProperty(JSON_PROPERTY_METHOD_ROLES)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public StaticRoleDefinitionAuthTemplate getMethodRoles() {
    return methodRoles;
  }


  @JsonProperty(JSON_PROPERTY_METHOD_ROLES)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setMethodRoles(StaticRoleDefinitionAuthTemplate methodRoles) {
    this.methodRoles = methodRoles;
  }


  /**
   * Return true if this AuthConfig object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    AuthConfig authConfig = (AuthConfig) o;
    return Objects.equals(this.functionAuthType, authConfig.functionAuthType) &&
        Objects.equals(this.functionAccessRules, authConfig.functionAccessRules) &&
        Objects.equals(this.methodAuthType, authConfig.methodAuthType) &&
        Objects.equals(this.methodRoles, authConfig.methodRoles);
  }

  @Override
  public int hashCode() {
    return Objects.hash(functionAuthType, functionAccessRules, methodAuthType, methodRoles);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class AuthConfig {\n");
    sb.append("    functionAuthType: ").append(toIndentedString(functionAuthType)).append("\n");
    sb.append("    functionAccessRules: ").append(toIndentedString(functionAccessRules)).append("\n");
    sb.append("    methodAuthType: ").append(toIndentedString(methodAuthType)).append("\n");
    sb.append("    methodRoles: ").append(toIndentedString(methodRoles)).append("\n");
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

