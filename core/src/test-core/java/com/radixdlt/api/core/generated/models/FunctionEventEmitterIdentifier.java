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
import com.radixdlt.api.core.generated.models.EventEmitterIdentifier;
import com.radixdlt.api.core.generated.models.EventEmitterIdentifierType;
import com.radixdlt.api.core.generated.models.FunctionEventEmitterIdentifier;
import com.radixdlt.api.core.generated.models.FunctionEventEmitterIdentifierAllOf;
import com.radixdlt.api.core.generated.models.MethodEventEmitterIdentifier;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


import com.radixdlt.api.core.generated.client.JSON;
/**
 * FunctionEventEmitterIdentifier
 */
@JsonPropertyOrder({
  FunctionEventEmitterIdentifier.JSON_PROPERTY_PACKAGE_ADDRESS,
  FunctionEventEmitterIdentifier.JSON_PROPERTY_BLUEPRINT_NAME
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
@JsonIgnoreProperties(
  value = "type", // ignore manually set type, it will be automatically generated by Jackson during serialization
  allowSetters = true // allows the type to be set during deserialization
)
@JsonTypeInfo(use = JsonTypeInfo.Id.NAME, include = JsonTypeInfo.As.PROPERTY, property = "type", visible = true)
@JsonSubTypes({
  @JsonSubTypes.Type(value = FunctionEventEmitterIdentifier.class, name = "Function"),
  @JsonSubTypes.Type(value = MethodEventEmitterIdentifier.class, name = "Method"),
})

public class FunctionEventEmitterIdentifier extends EventEmitterIdentifier {
  public static final String JSON_PROPERTY_PACKAGE_ADDRESS = "package_address";
  private String packageAddress;

  public static final String JSON_PROPERTY_BLUEPRINT_NAME = "blueprint_name";
  private String blueprintName;

  public FunctionEventEmitterIdentifier() { 
  }

  public FunctionEventEmitterIdentifier packageAddress(String packageAddress) {
    this.packageAddress = packageAddress;
    return this;
  }

   /**
   * The Bech32m-encoded human readable version of the package address
   * @return packageAddress
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The Bech32m-encoded human readable version of the package address")
  @JsonProperty(JSON_PROPERTY_PACKAGE_ADDRESS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getPackageAddress() {
    return packageAddress;
  }


  @JsonProperty(JSON_PROPERTY_PACKAGE_ADDRESS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setPackageAddress(String packageAddress) {
    this.packageAddress = packageAddress;
  }


  public FunctionEventEmitterIdentifier blueprintName(String blueprintName) {
    this.blueprintName = blueprintName;
    return this;
  }

   /**
   * The blueprint under the package which emitted the event.
   * @return blueprintName
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The blueprint under the package which emitted the event.")
  @JsonProperty(JSON_PROPERTY_BLUEPRINT_NAME)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getBlueprintName() {
    return blueprintName;
  }


  @JsonProperty(JSON_PROPERTY_BLUEPRINT_NAME)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setBlueprintName(String blueprintName) {
    this.blueprintName = blueprintName;
  }


  /**
   * Return true if this FunctionEventEmitterIdentifier object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    FunctionEventEmitterIdentifier functionEventEmitterIdentifier = (FunctionEventEmitterIdentifier) o;
    return Objects.equals(this.packageAddress, functionEventEmitterIdentifier.packageAddress) &&
        Objects.equals(this.blueprintName, functionEventEmitterIdentifier.blueprintName) &&
        super.equals(o);
  }

  @Override
  public int hashCode() {
    return Objects.hash(packageAddress, blueprintName, super.hashCode());
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class FunctionEventEmitterIdentifier {\n");
    sb.append("    ").append(toIndentedString(super.toString())).append("\n");
    sb.append("    packageAddress: ").append(toIndentedString(packageAddress)).append("\n");
    sb.append("    blueprintName: ").append(toIndentedString(blueprintName)).append("\n");
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
  mappings.put("Function", FunctionEventEmitterIdentifier.class);
  mappings.put("Method", MethodEventEmitterIdentifier.class);
  mappings.put("FunctionEventEmitterIdentifier", FunctionEventEmitterIdentifier.class);
  JSON.registerDiscriminator(FunctionEventEmitterIdentifier.class, "type", mappings);
}
}

