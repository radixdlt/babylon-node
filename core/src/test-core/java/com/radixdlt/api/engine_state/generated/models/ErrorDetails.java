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
import com.radixdlt.api.engine_state.generated.models.ErrorType;
import com.radixdlt.api.engine_state.generated.models.RequestedItemInvalidDetails;
import com.radixdlt.api.engine_state.generated.models.RequestedItemNotFoundDetails;
import com.radixdlt.api.engine_state.generated.models.StateVersionInFutureDetails;
import com.radixdlt.api.engine_state.generated.models.StateVersionInTooDistantPastDetails;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


import com.radixdlt.api.engine_state.generated.client.JSON;
/**
 * ErrorDetails
 */
@JsonPropertyOrder({
  ErrorDetails.JSON_PROPERTY_ERROR_TYPE
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
@JsonIgnoreProperties(
  value = "error_type", // ignore manually set error_type, it will be automatically generated by Jackson during serialization
  allowSetters = true // allows the error_type to be set during deserialization
)
@JsonTypeInfo(use = JsonTypeInfo.Id.NAME, include = JsonTypeInfo.As.PROPERTY, property = "error_type", visible = true)
@JsonSubTypes({
  @JsonSubTypes.Type(value = RequestedItemInvalidDetails.class, name = "RequestedItemInvalid"),
  @JsonSubTypes.Type(value = RequestedItemInvalidDetails.class, name = "RequestedItemInvalidDetails"),
  @JsonSubTypes.Type(value = RequestedItemNotFoundDetails.class, name = "RequestedItemNotFound"),
  @JsonSubTypes.Type(value = RequestedItemNotFoundDetails.class, name = "RequestedItemNotFoundDetails"),
  @JsonSubTypes.Type(value = StateVersionInFutureDetails.class, name = "StateVersionInFuture"),
  @JsonSubTypes.Type(value = StateVersionInFutureDetails.class, name = "StateVersionInFutureDetails"),
  @JsonSubTypes.Type(value = StateVersionInTooDistantPastDetails.class, name = "StateVersionInTooDistantPast"),
  @JsonSubTypes.Type(value = StateVersionInTooDistantPastDetails.class, name = "StateVersionInTooDistantPastDetails"),
})

public class ErrorDetails {
  public static final String JSON_PROPERTY_ERROR_TYPE = "error_type";
  private ErrorType errorType;

  public ErrorDetails() { 
  }

  public ErrorDetails errorType(ErrorType errorType) {
    this.errorType = errorType;
    return this;
  }

   /**
   * Get errorType
   * @return errorType
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_ERROR_TYPE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public ErrorType getErrorType() {
    return errorType;
  }


  @JsonProperty(JSON_PROPERTY_ERROR_TYPE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setErrorType(ErrorType errorType) {
    this.errorType = errorType;
  }


  /**
   * Return true if this ErrorDetails object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    ErrorDetails errorDetails = (ErrorDetails) o;
    return Objects.equals(this.errorType, errorDetails.errorType);
  }

  @Override
  public int hashCode() {
    return Objects.hash(errorType);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class ErrorDetails {\n");
    sb.append("    errorType: ").append(toIndentedString(errorType)).append("\n");
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
  mappings.put("RequestedItemInvalid", RequestedItemInvalidDetails.class);
  mappings.put("RequestedItemInvalidDetails", RequestedItemInvalidDetails.class);
  mappings.put("RequestedItemNotFound", RequestedItemNotFoundDetails.class);
  mappings.put("RequestedItemNotFoundDetails", RequestedItemNotFoundDetails.class);
  mappings.put("StateVersionInFuture", StateVersionInFutureDetails.class);
  mappings.put("StateVersionInFutureDetails", StateVersionInFutureDetails.class);
  mappings.put("StateVersionInTooDistantPast", StateVersionInTooDistantPastDetails.class);
  mappings.put("StateVersionInTooDistantPastDetails", StateVersionInTooDistantPastDetails.class);
  mappings.put("ErrorDetails", ErrorDetails.class);
  JSON.registerDiscriminator(ErrorDetails.class, "error_type", mappings);
}
}

