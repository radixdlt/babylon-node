/*
 * Radix Core API - Babylon
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.0.0
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
import com.radixdlt.api.core.generated.models.GenesisLedgerTransaction;
import com.radixdlt.api.core.generated.models.GenesisLedgerTransactionAllOf;
import com.radixdlt.api.core.generated.models.LedgerTransaction;
import com.radixdlt.api.core.generated.models.LedgerTransactionType;
import com.radixdlt.api.core.generated.models.RoundUpdateLedgerTransaction;
import com.radixdlt.api.core.generated.models.SystemTransaction;
import com.radixdlt.api.core.generated.models.UserLedgerTransaction;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


import com.radixdlt.api.core.generated.client.JSON;
/**
 * GenesisLedgerTransaction
 */
@JsonPropertyOrder({
  GenesisLedgerTransaction.JSON_PROPERTY_IS_FLASH,
  GenesisLedgerTransaction.JSON_PROPERTY_SYSTEM_TRANSACTION
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
@JsonIgnoreProperties(
  value = "type", // ignore manually set type, it will be automatically generated by Jackson during serialization
  allowSetters = true // allows the type to be set during deserialization
)
@JsonTypeInfo(use = JsonTypeInfo.Id.NAME, include = JsonTypeInfo.As.PROPERTY, property = "type", visible = true)
@JsonSubTypes({
  @JsonSubTypes.Type(value = GenesisLedgerTransaction.class, name = "Genesis"),
  @JsonSubTypes.Type(value = RoundUpdateLedgerTransaction.class, name = "RoundUpdate"),
  @JsonSubTypes.Type(value = UserLedgerTransaction.class, name = "User"),
})

public class GenesisLedgerTransaction extends LedgerTransaction {
  public static final String JSON_PROPERTY_IS_FLASH = "is_flash";
  private Boolean isFlash;

  public static final String JSON_PROPERTY_SYSTEM_TRANSACTION = "system_transaction";
  private SystemTransaction systemTransaction;

  public GenesisLedgerTransaction() { 
  }

  public GenesisLedgerTransaction isFlash(Boolean isFlash) {
    this.isFlash = isFlash;
    return this;
  }

   /**
   * The first genesis \&quot;transaction\&quot; flashes state into the database to prepare for the bootstrap transaction. Such a transaction does not have an associated &#x60;system_transaction&#x60; 
   * @return isFlash
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The first genesis \"transaction\" flashes state into the database to prepare for the bootstrap transaction. Such a transaction does not have an associated `system_transaction` ")
  @JsonProperty(JSON_PROPERTY_IS_FLASH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Boolean getIsFlash() {
    return isFlash;
  }


  @JsonProperty(JSON_PROPERTY_IS_FLASH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setIsFlash(Boolean isFlash) {
    this.isFlash = isFlash;
  }


  public GenesisLedgerTransaction systemTransaction(SystemTransaction systemTransaction) {
    this.systemTransaction = systemTransaction;
    return this;
  }

   /**
   * Get systemTransaction
   * @return systemTransaction
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "")
  @JsonProperty(JSON_PROPERTY_SYSTEM_TRANSACTION)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public SystemTransaction getSystemTransaction() {
    return systemTransaction;
  }


  @JsonProperty(JSON_PROPERTY_SYSTEM_TRANSACTION)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setSystemTransaction(SystemTransaction systemTransaction) {
    this.systemTransaction = systemTransaction;
  }


  /**
   * Return true if this GenesisLedgerTransaction object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    GenesisLedgerTransaction genesisLedgerTransaction = (GenesisLedgerTransaction) o;
    return Objects.equals(this.isFlash, genesisLedgerTransaction.isFlash) &&
        Objects.equals(this.systemTransaction, genesisLedgerTransaction.systemTransaction) &&
        super.equals(o);
  }

  @Override
  public int hashCode() {
    return Objects.hash(isFlash, systemTransaction, super.hashCode());
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class GenesisLedgerTransaction {\n");
    sb.append("    ").append(toIndentedString(super.toString())).append("\n");
    sb.append("    isFlash: ").append(toIndentedString(isFlash)).append("\n");
    sb.append("    systemTransaction: ").append(toIndentedString(systemTransaction)).append("\n");
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
  mappings.put("Genesis", GenesisLedgerTransaction.class);
  mappings.put("RoundUpdate", RoundUpdateLedgerTransaction.class);
  mappings.put("User", UserLedgerTransaction.class);
  mappings.put("GenesisLedgerTransaction", GenesisLedgerTransaction.class);
  JSON.registerDiscriminator(GenesisLedgerTransaction.class, "type", mappings);
}
}

