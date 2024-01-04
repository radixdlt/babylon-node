/*
 * Radix Core API - Babylon
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.0.4
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
import com.radixdlt.api.core.generated.models.BlueprintFunctionAuthorization;
import com.radixdlt.api.core.generated.models.BlueprintResolvedTypeReference;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * BlueprintFunctionInfo
 */
@JsonPropertyOrder({
  BlueprintFunctionInfo.JSON_PROPERTY_NAME,
  BlueprintFunctionInfo.JSON_PROPERTY_INPUT_TYPE_REFERENCE,
  BlueprintFunctionInfo.JSON_PROPERTY_OUTPUT_TYPE_REFERENCE,
  BlueprintFunctionInfo.JSON_PROPERTY_AUTHORIZATION
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class BlueprintFunctionInfo {
  public static final String JSON_PROPERTY_NAME = "name";
  private String name;

  public static final String JSON_PROPERTY_INPUT_TYPE_REFERENCE = "input_type_reference";
  private BlueprintResolvedTypeReference inputTypeReference;

  public static final String JSON_PROPERTY_OUTPUT_TYPE_REFERENCE = "output_type_reference";
  private BlueprintResolvedTypeReference outputTypeReference;

  public static final String JSON_PROPERTY_AUTHORIZATION = "authorization";
  private BlueprintFunctionAuthorization authorization;

  public BlueprintFunctionInfo() { 
  }

  public BlueprintFunctionInfo name(String name) {
    this.name = name;
    return this;
  }

   /**
   * Get name
   * @return name
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_NAME)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getName() {
    return name;
  }


  @JsonProperty(JSON_PROPERTY_NAME)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setName(String name) {
    this.name = name;
  }


  public BlueprintFunctionInfo inputTypeReference(BlueprintResolvedTypeReference inputTypeReference) {
    this.inputTypeReference = inputTypeReference;
    return this;
  }

   /**
   * Get inputTypeReference
   * @return inputTypeReference
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_INPUT_TYPE_REFERENCE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public BlueprintResolvedTypeReference getInputTypeReference() {
    return inputTypeReference;
  }


  @JsonProperty(JSON_PROPERTY_INPUT_TYPE_REFERENCE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setInputTypeReference(BlueprintResolvedTypeReference inputTypeReference) {
    this.inputTypeReference = inputTypeReference;
  }


  public BlueprintFunctionInfo outputTypeReference(BlueprintResolvedTypeReference outputTypeReference) {
    this.outputTypeReference = outputTypeReference;
    return this;
  }

   /**
   * Get outputTypeReference
   * @return outputTypeReference
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_OUTPUT_TYPE_REFERENCE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public BlueprintResolvedTypeReference getOutputTypeReference() {
    return outputTypeReference;
  }


  @JsonProperty(JSON_PROPERTY_OUTPUT_TYPE_REFERENCE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setOutputTypeReference(BlueprintResolvedTypeReference outputTypeReference) {
    this.outputTypeReference = outputTypeReference;
  }


  public BlueprintFunctionInfo authorization(BlueprintFunctionAuthorization authorization) {
    this.authorization = authorization;
    return this;
  }

   /**
   * Get authorization
   * @return authorization
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_AUTHORIZATION)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public BlueprintFunctionAuthorization getAuthorization() {
    return authorization;
  }


  @JsonProperty(JSON_PROPERTY_AUTHORIZATION)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setAuthorization(BlueprintFunctionAuthorization authorization) {
    this.authorization = authorization;
  }


  /**
   * Return true if this BlueprintFunctionInfo object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    BlueprintFunctionInfo blueprintFunctionInfo = (BlueprintFunctionInfo) o;
    return Objects.equals(this.name, blueprintFunctionInfo.name) &&
        Objects.equals(this.inputTypeReference, blueprintFunctionInfo.inputTypeReference) &&
        Objects.equals(this.outputTypeReference, blueprintFunctionInfo.outputTypeReference) &&
        Objects.equals(this.authorization, blueprintFunctionInfo.authorization);
  }

  @Override
  public int hashCode() {
    return Objects.hash(name, inputTypeReference, outputTypeReference, authorization);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class BlueprintFunctionInfo {\n");
    sb.append("    name: ").append(toIndentedString(name)).append("\n");
    sb.append("    inputTypeReference: ").append(toIndentedString(inputTypeReference)).append("\n");
    sb.append("    outputTypeReference: ").append(toIndentedString(outputTypeReference)).append("\n");
    sb.append("    authorization: ").append(toIndentedString(authorization)).append("\n");
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
