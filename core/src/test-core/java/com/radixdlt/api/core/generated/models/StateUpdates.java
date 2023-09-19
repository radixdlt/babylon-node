/*
 * Babylon Core API - RCnet v3.1
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  This version of the Core API belongs to the fourth release candidate of the Radix Babylon network (\"RCnet v3.1\").  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` are guaranteed to be forward compatible to Babylon mainnet launch (and beyond). We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code. 
 *
 * The version of the OpenAPI document: 0.5.1
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
import com.radixdlt.api.core.generated.models.CreatedSubstate;
import com.radixdlt.api.core.generated.models.DeletedSubstate;
import com.radixdlt.api.core.generated.models.EntityReference;
import com.radixdlt.api.core.generated.models.PartitionId;
import com.radixdlt.api.core.generated.models.UpdatedSubstate;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import java.util.ArrayList;
import java.util.List;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * Transaction state updates (only present if status is Succeeded or Failed)
 */
@ApiModel(description = "Transaction state updates (only present if status is Succeeded or Failed)")
@JsonPropertyOrder({
  StateUpdates.JSON_PROPERTY_DELETED_PARTITIONS,
  StateUpdates.JSON_PROPERTY_CREATED_SUBSTATES,
  StateUpdates.JSON_PROPERTY_UPDATED_SUBSTATES,
  StateUpdates.JSON_PROPERTY_DELETED_SUBSTATES,
  StateUpdates.JSON_PROPERTY_NEW_GLOBAL_ENTITIES
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class StateUpdates {
  public static final String JSON_PROPERTY_DELETED_PARTITIONS = "deleted_partitions";
  private List<PartitionId> deletedPartitions = new ArrayList<>();

  public static final String JSON_PROPERTY_CREATED_SUBSTATES = "created_substates";
  private List<CreatedSubstate> createdSubstates = new ArrayList<>();

  public static final String JSON_PROPERTY_UPDATED_SUBSTATES = "updated_substates";
  private List<UpdatedSubstate> updatedSubstates = new ArrayList<>();

  public static final String JSON_PROPERTY_DELETED_SUBSTATES = "deleted_substates";
  private List<DeletedSubstate> deletedSubstates = new ArrayList<>();

  public static final String JSON_PROPERTY_NEW_GLOBAL_ENTITIES = "new_global_entities";
  private List<EntityReference> newGlobalEntities = new ArrayList<>();

  public StateUpdates() { 
  }

  public StateUpdates deletedPartitions(List<PartitionId> deletedPartitions) {
    this.deletedPartitions = deletedPartitions;
    return this;
  }

  public StateUpdates addDeletedPartitionsItem(PartitionId deletedPartitionsItem) {
    this.deletedPartitions.add(deletedPartitionsItem);
    return this;
  }

   /**
   * Get deletedPartitions
   * @return deletedPartitions
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_DELETED_PARTITIONS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public List<PartitionId> getDeletedPartitions() {
    return deletedPartitions;
  }


  @JsonProperty(JSON_PROPERTY_DELETED_PARTITIONS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setDeletedPartitions(List<PartitionId> deletedPartitions) {
    this.deletedPartitions = deletedPartitions;
  }


  public StateUpdates createdSubstates(List<CreatedSubstate> createdSubstates) {
    this.createdSubstates = createdSubstates;
    return this;
  }

  public StateUpdates addCreatedSubstatesItem(CreatedSubstate createdSubstatesItem) {
    this.createdSubstates.add(createdSubstatesItem);
    return this;
  }

   /**
   * Get createdSubstates
   * @return createdSubstates
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_CREATED_SUBSTATES)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public List<CreatedSubstate> getCreatedSubstates() {
    return createdSubstates;
  }


  @JsonProperty(JSON_PROPERTY_CREATED_SUBSTATES)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setCreatedSubstates(List<CreatedSubstate> createdSubstates) {
    this.createdSubstates = createdSubstates;
  }


  public StateUpdates updatedSubstates(List<UpdatedSubstate> updatedSubstates) {
    this.updatedSubstates = updatedSubstates;
    return this;
  }

  public StateUpdates addUpdatedSubstatesItem(UpdatedSubstate updatedSubstatesItem) {
    this.updatedSubstates.add(updatedSubstatesItem);
    return this;
  }

   /**
   * Get updatedSubstates
   * @return updatedSubstates
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_UPDATED_SUBSTATES)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public List<UpdatedSubstate> getUpdatedSubstates() {
    return updatedSubstates;
  }


  @JsonProperty(JSON_PROPERTY_UPDATED_SUBSTATES)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setUpdatedSubstates(List<UpdatedSubstate> updatedSubstates) {
    this.updatedSubstates = updatedSubstates;
  }


  public StateUpdates deletedSubstates(List<DeletedSubstate> deletedSubstates) {
    this.deletedSubstates = deletedSubstates;
    return this;
  }

  public StateUpdates addDeletedSubstatesItem(DeletedSubstate deletedSubstatesItem) {
    this.deletedSubstates.add(deletedSubstatesItem);
    return this;
  }

   /**
   * Get deletedSubstates
   * @return deletedSubstates
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_DELETED_SUBSTATES)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public List<DeletedSubstate> getDeletedSubstates() {
    return deletedSubstates;
  }


  @JsonProperty(JSON_PROPERTY_DELETED_SUBSTATES)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setDeletedSubstates(List<DeletedSubstate> deletedSubstates) {
    this.deletedSubstates = deletedSubstates;
  }


  public StateUpdates newGlobalEntities(List<EntityReference> newGlobalEntities) {
    this.newGlobalEntities = newGlobalEntities;
    return this;
  }

  public StateUpdates addNewGlobalEntitiesItem(EntityReference newGlobalEntitiesItem) {
    this.newGlobalEntities.add(newGlobalEntitiesItem);
    return this;
  }

   /**
   * Get newGlobalEntities
   * @return newGlobalEntities
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_NEW_GLOBAL_ENTITIES)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public List<EntityReference> getNewGlobalEntities() {
    return newGlobalEntities;
  }


  @JsonProperty(JSON_PROPERTY_NEW_GLOBAL_ENTITIES)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setNewGlobalEntities(List<EntityReference> newGlobalEntities) {
    this.newGlobalEntities = newGlobalEntities;
  }


  /**
   * Return true if this StateUpdates object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    StateUpdates stateUpdates = (StateUpdates) o;
    return Objects.equals(this.deletedPartitions, stateUpdates.deletedPartitions) &&
        Objects.equals(this.createdSubstates, stateUpdates.createdSubstates) &&
        Objects.equals(this.updatedSubstates, stateUpdates.updatedSubstates) &&
        Objects.equals(this.deletedSubstates, stateUpdates.deletedSubstates) &&
        Objects.equals(this.newGlobalEntities, stateUpdates.newGlobalEntities);
  }

  @Override
  public int hashCode() {
    return Objects.hash(deletedPartitions, createdSubstates, updatedSubstates, deletedSubstates, newGlobalEntities);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class StateUpdates {\n");
    sb.append("    deletedPartitions: ").append(toIndentedString(deletedPartitions)).append("\n");
    sb.append("    createdSubstates: ").append(toIndentedString(createdSubstates)).append("\n");
    sb.append("    updatedSubstates: ").append(toIndentedString(updatedSubstates)).append("\n");
    sb.append("    deletedSubstates: ").append(toIndentedString(deletedSubstates)).append("\n");
    sb.append("    newGlobalEntities: ").append(toIndentedString(newGlobalEntities)).append("\n");
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

