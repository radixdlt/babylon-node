/*
 * Engine State API - Babylon (Anemone)
 * **This API is currently in Beta**  This specification may experience breaking changes as part of Babylon Node releases. Such changes will be clearly mentioned in the [babylon-node release notes](https://github.com/radixdlt/babylon-node/releases). We advise against using this API for business-critical integrations before the `version` indicated above becomes stable, which is expected in Q4 of 2024.  This API provides a complete view of the current ledger state, operating at a relatively low level (i.e. returning Entities' data and type information in a generic way, without interpreting specifics of different native or custom components).  It mirrors how the Radix Engine views the ledger state in its \"System\" layer, and thus can be useful for Scrypto developers, who need to inspect how the Engine models and stores their application's state, or how an interface / authentication scheme of another component looks like. 
 *
 * The version of the OpenAPI document: v0.1-beta
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
 * BlueprintRoleInfo
 */
@JsonPropertyOrder({
  BlueprintRoleInfo.JSON_PROPERTY_KEY,
  BlueprintRoleInfo.JSON_PROPERTY_UPDATER_ROLE_KEYS
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class BlueprintRoleInfo {
  public static final String JSON_PROPERTY_KEY = "key";
  private String key;

  public static final String JSON_PROPERTY_UPDATER_ROLE_KEYS = "updater_role_keys";
  private List<String> updaterRoleKeys = new ArrayList<>();

  public BlueprintRoleInfo() { 
  }

  public BlueprintRoleInfo key(String key) {
    this.key = key;
    return this;
  }

   /**
   * Identifier of a role.
   * @return key
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "Identifier of a role.")
  @JsonProperty(JSON_PROPERTY_KEY)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getKey() {
    return key;
  }


  @JsonProperty(JSON_PROPERTY_KEY)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setKey(String key) {
    this.key = key;
  }


  public BlueprintRoleInfo updaterRoleKeys(List<String> updaterRoleKeys) {
    this.updaterRoleKeys = updaterRoleKeys;
    return this;
  }

  public BlueprintRoleInfo addUpdaterRoleKeysItem(String updaterRoleKeysItem) {
    this.updaterRoleKeys.add(updaterRoleKeysItem);
    return this;
  }

   /**
   * Get updaterRoleKeys
   * @return updaterRoleKeys
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_UPDATER_ROLE_KEYS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public List<String> getUpdaterRoleKeys() {
    return updaterRoleKeys;
  }


  @JsonProperty(JSON_PROPERTY_UPDATER_ROLE_KEYS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setUpdaterRoleKeys(List<String> updaterRoleKeys) {
    this.updaterRoleKeys = updaterRoleKeys;
  }


  /**
   * Return true if this BlueprintRoleInfo object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    BlueprintRoleInfo blueprintRoleInfo = (BlueprintRoleInfo) o;
    return Objects.equals(this.key, blueprintRoleInfo.key) &&
        Objects.equals(this.updaterRoleKeys, blueprintRoleInfo.updaterRoleKeys);
  }

  @Override
  public int hashCode() {
    return Objects.hash(key, updaterRoleKeys);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class BlueprintRoleInfo {\n");
    sb.append("    key: ").append(toIndentedString(key)).append("\n");
    sb.append("    updaterRoleKeys: ").append(toIndentedString(updaterRoleKeys)).append("\n");
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
