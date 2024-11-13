/*
 * Engine State API (Beta)
 * **This API is currently in Beta**  This specification may experience breaking changes as part of Babylon Node releases. Such changes will be clearly mentioned in the [babylon-node release notes](https://github.com/radixdlt/babylon-node/releases). We advise against using this API for business-critical integrations before the `version` indicated above becomes stable, which is expected in Q4 of 2024.  This API provides a complete view of the current ledger state, operating at a relatively low level (i.e. returning Entities' data and type information in a generic way, without interpreting specifics of different native or custom components).  It mirrors how the Radix Engine views the ledger state in its \"System\" layer, and thus can be useful for Scrypto developers, who need to inspect how the Engine models and stores their application's state, or how an interface / authentication scheme of another component looks like. 
 *
 * The version of the OpenAPI document: v1.3.0
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
import com.radixdlt.api.engine_state.generated.models.AllOfBasicRequirement;
import com.radixdlt.api.engine_state.generated.models.AllOfBasicRequirementAllOf;
import com.radixdlt.api.engine_state.generated.models.AmountOfBasicRequirement;
import com.radixdlt.api.engine_state.generated.models.AnyOfBasicRequirement;
import com.radixdlt.api.engine_state.generated.models.BasicRequirement;
import com.radixdlt.api.engine_state.generated.models.BasicRequirementType;
import com.radixdlt.api.engine_state.generated.models.CountOfBasicRequirement;
import com.radixdlt.api.engine_state.generated.models.RequireBasicRequirement;
import com.radixdlt.api.engine_state.generated.models.Requirement;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import java.util.ArrayList;
import java.util.List;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


import com.radixdlt.api.engine_state.generated.client.JSON;
/**
 * AllOfBasicRequirement
 */
@JsonPropertyOrder({
  AllOfBasicRequirement.JSON_PROPERTY_LIST
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
@JsonIgnoreProperties(
  value = "type", // ignore manually set type, it will be automatically generated by Jackson during serialization
  allowSetters = true // allows the type to be set during deserialization
)
@JsonTypeInfo(use = JsonTypeInfo.Id.NAME, include = JsonTypeInfo.As.PROPERTY, property = "type", visible = true)
@JsonSubTypes({
  @JsonSubTypes.Type(value = AllOfBasicRequirement.class, name = "AllOf"),
  @JsonSubTypes.Type(value = AmountOfBasicRequirement.class, name = "AmountOf"),
  @JsonSubTypes.Type(value = AnyOfBasicRequirement.class, name = "AnyOf"),
  @JsonSubTypes.Type(value = CountOfBasicRequirement.class, name = "CountOf"),
  @JsonSubTypes.Type(value = RequireBasicRequirement.class, name = "Require"),
})

public class AllOfBasicRequirement extends BasicRequirement {
  public static final String JSON_PROPERTY_LIST = "list";
  private List<Requirement> _list = new ArrayList<>();

  public AllOfBasicRequirement() { 
  }

  public AllOfBasicRequirement _list(List<Requirement> _list) {
    this._list = _list;
    return this;
  }

  public AllOfBasicRequirement addListItem(Requirement _listItem) {
    this._list.add(_listItem);
    return this;
  }

   /**
   * Get _list
   * @return _list
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_LIST)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public List<Requirement> getList() {
    return _list;
  }


  @JsonProperty(JSON_PROPERTY_LIST)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setList(List<Requirement> _list) {
    this._list = _list;
  }


  /**
   * Return true if this AllOfBasicRequirement object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    AllOfBasicRequirement allOfBasicRequirement = (AllOfBasicRequirement) o;
    return Objects.equals(this._list, allOfBasicRequirement._list) &&
        super.equals(o);
  }

  @Override
  public int hashCode() {
    return Objects.hash(_list, super.hashCode());
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class AllOfBasicRequirement {\n");
    sb.append("    ").append(toIndentedString(super.toString())).append("\n");
    sb.append("    _list: ").append(toIndentedString(_list)).append("\n");
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
  mappings.put("AllOf", AllOfBasicRequirement.class);
  mappings.put("AmountOf", AmountOfBasicRequirement.class);
  mappings.put("AnyOf", AnyOfBasicRequirement.class);
  mappings.put("CountOf", CountOfBasicRequirement.class);
  mappings.put("Require", RequireBasicRequirement.class);
  mappings.put("AllOfBasicRequirement", AllOfBasicRequirement.class);
  JSON.registerDiscriminator(AllOfBasicRequirement.class, "type", mappings);
}
}

