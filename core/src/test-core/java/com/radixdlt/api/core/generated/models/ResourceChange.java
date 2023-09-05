/*
 * Babylon Core API - RCnet v3.1
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  This version of the Core API belongs to the fourth release candidate of the Radix Babylon network (\"RCnet v3.1\").  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` are guaranteed to be forward compatible to Babylon mainnet launch (and beyond). We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code. 
 *
 * The version of the OpenAPI document: 0.5.0
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
import com.radixdlt.api.core.generated.models.EntityReference;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * ResourceChange
 */
@JsonPropertyOrder({
  ResourceChange.JSON_PROPERTY_RESOURCE_ADDRESS,
  ResourceChange.JSON_PROPERTY_COMPONENT_ENTITY,
  ResourceChange.JSON_PROPERTY_VAULT_ENTITY,
  ResourceChange.JSON_PROPERTY_AMOUNT
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class ResourceChange {
  public static final String JSON_PROPERTY_RESOURCE_ADDRESS = "resource_address";
  private String resourceAddress;

  public static final String JSON_PROPERTY_COMPONENT_ENTITY = "component_entity";
  private EntityReference componentEntity;

  public static final String JSON_PROPERTY_VAULT_ENTITY = "vault_entity";
  private EntityReference vaultEntity;

  public static final String JSON_PROPERTY_AMOUNT = "amount";
  private String amount;

  public ResourceChange() { 
  }

  public ResourceChange resourceAddress(String resourceAddress) {
    this.resourceAddress = resourceAddress;
    return this;
  }

   /**
   * The Bech32m-encoded human readable version of the resource address
   * @return resourceAddress
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The Bech32m-encoded human readable version of the resource address")
  @JsonProperty(JSON_PROPERTY_RESOURCE_ADDRESS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getResourceAddress() {
    return resourceAddress;
  }


  @JsonProperty(JSON_PROPERTY_RESOURCE_ADDRESS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setResourceAddress(String resourceAddress) {
    this.resourceAddress = resourceAddress;
  }


  public ResourceChange componentEntity(EntityReference componentEntity) {
    this.componentEntity = componentEntity;
    return this;
  }

   /**
   * Get componentEntity
   * @return componentEntity
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_COMPONENT_ENTITY)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public EntityReference getComponentEntity() {
    return componentEntity;
  }


  @JsonProperty(JSON_PROPERTY_COMPONENT_ENTITY)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setComponentEntity(EntityReference componentEntity) {
    this.componentEntity = componentEntity;
  }


  public ResourceChange vaultEntity(EntityReference vaultEntity) {
    this.vaultEntity = vaultEntity;
    return this;
  }

   /**
   * Get vaultEntity
   * @return vaultEntity
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_VAULT_ENTITY)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public EntityReference getVaultEntity() {
    return vaultEntity;
  }


  @JsonProperty(JSON_PROPERTY_VAULT_ENTITY)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setVaultEntity(EntityReference vaultEntity) {
    this.vaultEntity = vaultEntity;
  }


  public ResourceChange amount(String amount) {
    this.amount = amount;
    return this;
  }

   /**
   * The string-encoded decimal representing the XRD amount put or taken from the vault. A decimal is formed of some signed integer &#x60;m&#x60; of attos (&#x60;10^(-18)&#x60;) units, where &#x60;-2^(192 - 1) &lt;&#x3D; m &lt; 2^(192 - 1)&#x60;. 
   * @return amount
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The string-encoded decimal representing the XRD amount put or taken from the vault. A decimal is formed of some signed integer `m` of attos (`10^(-18)`) units, where `-2^(192 - 1) <= m < 2^(192 - 1)`. ")
  @JsonProperty(JSON_PROPERTY_AMOUNT)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getAmount() {
    return amount;
  }


  @JsonProperty(JSON_PROPERTY_AMOUNT)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setAmount(String amount) {
    this.amount = amount;
  }


  /**
   * Return true if this ResourceChange object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    ResourceChange resourceChange = (ResourceChange) o;
    return Objects.equals(this.resourceAddress, resourceChange.resourceAddress) &&
        Objects.equals(this.componentEntity, resourceChange.componentEntity) &&
        Objects.equals(this.vaultEntity, resourceChange.vaultEntity) &&
        Objects.equals(this.amount, resourceChange.amount);
  }

  @Override
  public int hashCode() {
    return Objects.hash(resourceAddress, componentEntity, vaultEntity, amount);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class ResourceChange {\n");
    sb.append("    resourceAddress: ").append(toIndentedString(resourceAddress)).append("\n");
    sb.append("    componentEntity: ").append(toIndentedString(componentEntity)).append("\n");
    sb.append("    vaultEntity: ").append(toIndentedString(vaultEntity)).append("\n");
    sb.append("    amount: ").append(toIndentedString(amount)).append("\n");
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

