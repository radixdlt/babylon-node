/*
 * Babylon Core API - RCnet v3
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  This version of the Core API belongs to the second release candidate of the Radix Babylon network (\"RCnet v3\").  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` are guaranteed to be forward compatible to Babylon mainnet launch (and beyond). We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code. 
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
import com.radixdlt.api.core.generated.models.LedgerStateSummary;
import com.radixdlt.api.core.generated.models.StateResourceManager;
import com.radixdlt.api.core.generated.models.Substate;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * StateResourceResponse
 */
@JsonPropertyOrder({
  StateResourceResponse.JSON_PROPERTY_AT_LEDGER_STATE,
  StateResourceResponse.JSON_PROPERTY_MANAGER,
  StateResourceResponse.JSON_PROPERTY_OWNER_ROLE
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class StateResourceResponse {
  public static final String JSON_PROPERTY_AT_LEDGER_STATE = "at_ledger_state";
  private LedgerStateSummary atLedgerState;

  public static final String JSON_PROPERTY_MANAGER = "manager";
  private StateResourceManager manager;

  public static final String JSON_PROPERTY_OWNER_ROLE = "owner_role";
  private Substate ownerRole;

  public StateResourceResponse() { 
  }

  public StateResourceResponse atLedgerState(LedgerStateSummary atLedgerState) {
    this.atLedgerState = atLedgerState;
    return this;
  }

   /**
   * Get atLedgerState
   * @return atLedgerState
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_AT_LEDGER_STATE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public LedgerStateSummary getAtLedgerState() {
    return atLedgerState;
  }


  @JsonProperty(JSON_PROPERTY_AT_LEDGER_STATE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setAtLedgerState(LedgerStateSummary atLedgerState) {
    this.atLedgerState = atLedgerState;
  }


  public StateResourceResponse manager(StateResourceManager manager) {
    this.manager = manager;
    return this;
  }

   /**
   * Get manager
   * @return manager
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_MANAGER)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public StateResourceManager getManager() {
    return manager;
  }


  @JsonProperty(JSON_PROPERTY_MANAGER)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setManager(StateResourceManager manager) {
    this.manager = manager;
  }


  public StateResourceResponse ownerRole(Substate ownerRole) {
    this.ownerRole = ownerRole;
    return this;
  }

   /**
   * Get ownerRole
   * @return ownerRole
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_OWNER_ROLE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Substate getOwnerRole() {
    return ownerRole;
  }


  @JsonProperty(JSON_PROPERTY_OWNER_ROLE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setOwnerRole(Substate ownerRole) {
    this.ownerRole = ownerRole;
  }


  /**
   * Return true if this StateResourceResponse object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    StateResourceResponse stateResourceResponse = (StateResourceResponse) o;
    return Objects.equals(this.atLedgerState, stateResourceResponse.atLedgerState) &&
        Objects.equals(this.manager, stateResourceResponse.manager) &&
        Objects.equals(this.ownerRole, stateResourceResponse.ownerRole);
  }

  @Override
  public int hashCode() {
    return Objects.hash(atLedgerState, manager, ownerRole);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class StateResourceResponse {\n");
    sb.append("    atLedgerState: ").append(toIndentedString(atLedgerState)).append("\n");
    sb.append("    manager: ").append(toIndentedString(manager)).append("\n");
    sb.append("    ownerRole: ").append(toIndentedString(ownerRole)).append("\n");
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

