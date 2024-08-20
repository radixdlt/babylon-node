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
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonTypeName;
import com.fasterxml.jackson.annotation.JsonValue;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import java.util.ArrayList;
import java.util.List;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * Only the listed roles have access.
 */
@ApiModel(description = "Only the listed roles have access.")
@JsonPropertyOrder({
  ByRolesBlueprintMethodAuthorizationAllOf.JSON_PROPERTY_ROLE_KEYS
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class ByRolesBlueprintMethodAuthorizationAllOf {
  public static final String JSON_PROPERTY_ROLE_KEYS = "role_keys";
  private List<String> roleKeys = new ArrayList<>();

  public ByRolesBlueprintMethodAuthorizationAllOf() { 
  }

  public ByRolesBlueprintMethodAuthorizationAllOf roleKeys(List<String> roleKeys) {
    this.roleKeys = roleKeys;
    return this;
  }

  public ByRolesBlueprintMethodAuthorizationAllOf addRoleKeysItem(String roleKeysItem) {
    this.roleKeys.add(roleKeysItem);
    return this;
  }

   /**
   * Get roleKeys
   * @return roleKeys
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_ROLE_KEYS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public List<String> getRoleKeys() {
    return roleKeys;
  }


  @JsonProperty(JSON_PROPERTY_ROLE_KEYS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setRoleKeys(List<String> roleKeys) {
    this.roleKeys = roleKeys;
  }


  /**
   * Return true if this ByRolesBlueprintMethodAuthorization_allOf object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    ByRolesBlueprintMethodAuthorizationAllOf byRolesBlueprintMethodAuthorizationAllOf = (ByRolesBlueprintMethodAuthorizationAllOf) o;
    return Objects.equals(this.roleKeys, byRolesBlueprintMethodAuthorizationAllOf.roleKeys);
  }

  @Override
  public int hashCode() {
    return Objects.hash(roleKeys);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class ByRolesBlueprintMethodAuthorizationAllOf {\n");
    sb.append("    roleKeys: ").append(toIndentedString(roleKeys)).append("\n");
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

