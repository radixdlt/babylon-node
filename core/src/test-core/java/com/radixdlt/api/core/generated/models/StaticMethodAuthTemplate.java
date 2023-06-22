/*
 * Babylon Core API - RCnet V2
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  This version of the Core API belongs to the first release candidate of the Radix Babylon network (\"RCnet-V1\").  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` are guaranteed to be forward compatible to Babylon mainnet launch (and beyond). We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  We give no guarantees that other endpoints will not change before Babylon mainnet launch, although changes are expected to be minimal. 
 *
 * The version of the OpenAPI document: 0.4.0
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
import com.radixdlt.api.core.generated.models.MethodPermission;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * StaticMethodAuthTemplate
 */
@JsonPropertyOrder({
  StaticMethodAuthTemplate.JSON_PROPERTY_AUTH,
  StaticMethodAuthTemplate.JSON_PROPERTY_OUTER_AUTH
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class StaticMethodAuthTemplate {
  public static final String JSON_PROPERTY_AUTH = "auth";
  private Map<String, MethodPermission> auth = new HashMap<>();

  public static final String JSON_PROPERTY_OUTER_AUTH = "outer_auth";
  private Map<String, MethodPermission> outerAuth = new HashMap<>();

  public StaticMethodAuthTemplate() { 
  }

  public StaticMethodAuthTemplate auth(Map<String, MethodPermission> auth) {
    this.auth = auth;
    return this;
  }

  public StaticMethodAuthTemplate putAuthItem(String key, MethodPermission authItem) {
    this.auth.put(key, authItem);
    return this;
  }

   /**
   * A map from a method identifier to MethodPermission
   * @return auth
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "A map from a method identifier to MethodPermission")
  @JsonProperty(JSON_PROPERTY_AUTH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Map<String, MethodPermission> getAuth() {
    return auth;
  }


  @JsonProperty(JSON_PROPERTY_AUTH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setAuth(Map<String, MethodPermission> auth) {
    this.auth = auth;
  }


  public StaticMethodAuthTemplate outerAuth(Map<String, MethodPermission> outerAuth) {
    this.outerAuth = outerAuth;
    return this;
  }

  public StaticMethodAuthTemplate putOuterAuthItem(String key, MethodPermission outerAuthItem) {
    this.outerAuth.put(key, outerAuthItem);
    return this;
  }

   /**
   * A map from a method identifier to MethodPermission
   * @return outerAuth
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "A map from a method identifier to MethodPermission")
  @JsonProperty(JSON_PROPERTY_OUTER_AUTH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Map<String, MethodPermission> getOuterAuth() {
    return outerAuth;
  }


  @JsonProperty(JSON_PROPERTY_OUTER_AUTH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setOuterAuth(Map<String, MethodPermission> outerAuth) {
    this.outerAuth = outerAuth;
  }


  /**
   * Return true if this StaticMethodAuthTemplate object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    StaticMethodAuthTemplate staticMethodAuthTemplate = (StaticMethodAuthTemplate) o;
    return Objects.equals(this.auth, staticMethodAuthTemplate.auth) &&
        Objects.equals(this.outerAuth, staticMethodAuthTemplate.outerAuth);
  }

  @Override
  public int hashCode() {
    return Objects.hash(auth, outerAuth);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class StaticMethodAuthTemplate {\n");
    sb.append("    auth: ").append(toIndentedString(auth)).append("\n");
    sb.append("    outerAuth: ").append(toIndentedString(outerAuth)).append("\n");
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

