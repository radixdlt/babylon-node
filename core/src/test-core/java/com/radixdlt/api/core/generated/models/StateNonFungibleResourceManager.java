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
import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonSubTypes;
import com.fasterxml.jackson.annotation.JsonTypeInfo;
import com.fasterxml.jackson.annotation.JsonTypeName;
import com.fasterxml.jackson.annotation.JsonValue;
import com.radixdlt.api.core.generated.models.ResourceType;
import com.radixdlt.api.core.generated.models.StateFungibleResourceManager;
import com.radixdlt.api.core.generated.models.StateNonFungibleResourceManager;
import com.radixdlt.api.core.generated.models.StateNonFungibleResourceManagerAllOf;
import com.radixdlt.api.core.generated.models.StateResourceManager;
import com.radixdlt.api.core.generated.models.Substate;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


import com.radixdlt.api.core.generated.client.JSON;
/**
 * StateNonFungibleResourceManager
 */
@JsonPropertyOrder({
  StateNonFungibleResourceManager.JSON_PROPERTY_ID_TYPE,
  StateNonFungibleResourceManager.JSON_PROPERTY_TOTAL_SUPPLY,
  StateNonFungibleResourceManager.JSON_PROPERTY_MUTABLE_FIELDS
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
@JsonIgnoreProperties(
  value = "resource_type", // ignore manually set resource_type, it will be automatically generated by Jackson during serialization
  allowSetters = true // allows the resource_type to be set during deserialization
)
@JsonTypeInfo(use = JsonTypeInfo.Id.NAME, include = JsonTypeInfo.As.PROPERTY, property = "resource_type", visible = true)
@JsonSubTypes({
  @JsonSubTypes.Type(value = StateFungibleResourceManager.class, name = "Fungible"),
  @JsonSubTypes.Type(value = StateNonFungibleResourceManager.class, name = "NonFungible"),
})

public class StateNonFungibleResourceManager extends StateResourceManager {
  public static final String JSON_PROPERTY_ID_TYPE = "id_type";
  private Substate idType;

  public static final String JSON_PROPERTY_TOTAL_SUPPLY = "total_supply";
  private Substate totalSupply;

  public static final String JSON_PROPERTY_MUTABLE_FIELDS = "mutable_fields";
  private Substate mutableFields;

  public StateNonFungibleResourceManager() { 
  }

  public StateNonFungibleResourceManager idType(Substate idType) {
    this.idType = idType;
    return this;
  }

   /**
   * Get idType
   * @return idType
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_ID_TYPE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Substate getIdType() {
    return idType;
  }


  @JsonProperty(JSON_PROPERTY_ID_TYPE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setIdType(Substate idType) {
    this.idType = idType;
  }


  public StateNonFungibleResourceManager totalSupply(Substate totalSupply) {
    this.totalSupply = totalSupply;
    return this;
  }

   /**
   * Get totalSupply
   * @return totalSupply
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_TOTAL_SUPPLY)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Substate getTotalSupply() {
    return totalSupply;
  }


  @JsonProperty(JSON_PROPERTY_TOTAL_SUPPLY)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setTotalSupply(Substate totalSupply) {
    this.totalSupply = totalSupply;
  }


  public StateNonFungibleResourceManager mutableFields(Substate mutableFields) {
    this.mutableFields = mutableFields;
    return this;
  }

   /**
   * Get mutableFields
   * @return mutableFields
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_MUTABLE_FIELDS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Substate getMutableFields() {
    return mutableFields;
  }


  @JsonProperty(JSON_PROPERTY_MUTABLE_FIELDS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setMutableFields(Substate mutableFields) {
    this.mutableFields = mutableFields;
  }


  /**
   * Return true if this StateNonFungibleResourceManager object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    StateNonFungibleResourceManager stateNonFungibleResourceManager = (StateNonFungibleResourceManager) o;
    return Objects.equals(this.idType, stateNonFungibleResourceManager.idType) &&
        Objects.equals(this.totalSupply, stateNonFungibleResourceManager.totalSupply) &&
        Objects.equals(this.mutableFields, stateNonFungibleResourceManager.mutableFields) &&
        super.equals(o);
  }

  @Override
  public int hashCode() {
    return Objects.hash(idType, totalSupply, mutableFields, super.hashCode());
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class StateNonFungibleResourceManager {\n");
    sb.append("    ").append(toIndentedString(super.toString())).append("\n");
    sb.append("    idType: ").append(toIndentedString(idType)).append("\n");
    sb.append("    totalSupply: ").append(toIndentedString(totalSupply)).append("\n");
    sb.append("    mutableFields: ").append(toIndentedString(mutableFields)).append("\n");
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
  mappings.put("Fungible", StateFungibleResourceManager.class);
  mappings.put("NonFungible", StateNonFungibleResourceManager.class);
  mappings.put("StateNonFungibleResourceManager", StateNonFungibleResourceManager.class);
  JSON.registerDiscriminator(StateNonFungibleResourceManager.class, "resource_type", mappings);
}
}

