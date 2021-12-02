/*
 * Radix Gateway API
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 0.9.0
 * 
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */


package com.radixdlt.api.gateway.openapitools.model;

import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;
import com.fasterxml.jackson.annotation.JsonSubTypes;
import com.fasterxml.jackson.annotation.JsonTypeInfo;
import com.radixdlt.api.gateway.openapitools.JSON;
import io.swagger.annotations.ApiModelProperty;

import java.util.HashMap;
import java.util.Map;
import java.util.Objects;

/**
 * InvalidAccountAddressError
 */
@JsonPropertyOrder({
  InvalidAccountAddressError.JSON_PROPERTY_INVALID_ACCOUNT_ADDRESS
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen", date = "2021-12-01T22:57:23.640286-06:00[America/Chicago]")
@JsonTypeInfo(use = JsonTypeInfo.Id.NAME, include = JsonTypeInfo.As.EXISTING_PROPERTY, property = "type", visible = true)
@JsonSubTypes({
  @JsonSubTypes.Type(value = BelowMinimumStakeError.class, name = "BelowMinimumStakeError"),
  @JsonSubTypes.Type(value = CouldNotConstructFeesError.class, name = "CouldNotConstructFeesError"),
  @JsonSubTypes.Type(value = InvalidAccountAddressError.class, name = "InvalidAccountAddressError"),
  @JsonSubTypes.Type(value = InvalidPublicKeyError.class, name = "InvalidPublicKeyError"),
  @JsonSubTypes.Type(value = InvalidTokenRRIError.class, name = "InvalidTokenRRIError"),
  @JsonSubTypes.Type(value = InvalidTokenSymbolError.class, name = "InvalidTokenSymbolError"),
  @JsonSubTypes.Type(value = InvalidValidatorAddressError.class, name = "InvalidValidatorAddressError"),
  @JsonSubTypes.Type(value = MessageTooLongError.class, name = "MessageTooLongError"),
  @JsonSubTypes.Type(value = NotEnoughResourcesError.class, name = "NotEnoughResourcesError"),
  @JsonSubTypes.Type(value = CannotStakeError.class, name = "NotValidatorOwnerError"),
  @JsonSubTypes.Type(value = TokenNotFoundError.class, name = "TokenNotFound"),
})

public class InvalidAccountAddressError extends java.lang.Error {
  public static final String JSON_PROPERTY_INVALID_ACCOUNT_ADDRESS = "invalid_account_address";
  private String invalidAccountAddress;


  public InvalidAccountAddressError invalidAccountAddress(String invalidAccountAddress) {
    this.invalidAccountAddress = invalidAccountAddress;
    return this;
  }

   /**
   * Get invalidAccountAddress
   * @return invalidAccountAddress
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_INVALID_ACCOUNT_ADDRESS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getInvalidAccountAddress() {
    return invalidAccountAddress;
  }


  @JsonProperty(JSON_PROPERTY_INVALID_ACCOUNT_ADDRESS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setInvalidAccountAddress(String invalidAccountAddress) {
    this.invalidAccountAddress = invalidAccountAddress;
  }


  /**
   * Return true if this InvalidAccountAddressError object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    InvalidAccountAddressError invalidAccountAddressError = (InvalidAccountAddressError) o;
    return Objects.equals(this.invalidAccountAddress, invalidAccountAddressError.invalidAccountAddress) &&
        super.equals(o);
  }

  @Override
  public int hashCode() {
    return Objects.hash(invalidAccountAddress, super.hashCode());
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class InvalidAccountAddressError {\n");
    sb.append("    ").append(toIndentedString(super.toString())).append("\n");
    sb.append("    invalidAccountAddress: ").append(toIndentedString(invalidAccountAddress)).append("\n");
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
  mappings.put("BelowMinimumStakeError", BelowMinimumStakeError.class);
  mappings.put("CouldNotConstructFeesError", CouldNotConstructFeesError.class);
  mappings.put("InvalidAccountAddressError", InvalidAccountAddressError.class);
  mappings.put("InvalidPublicKeyError", InvalidPublicKeyError.class);
  mappings.put("InvalidTokenRRIError", InvalidTokenRRIError.class);
  mappings.put("InvalidTokenSymbolError", InvalidTokenSymbolError.class);
  mappings.put("InvalidValidatorAddressError", InvalidValidatorAddressError.class);
  mappings.put("MessageTooLongError", MessageTooLongError.class);
  mappings.put("NotEnoughResourcesError", NotEnoughResourcesError.class);
  mappings.put("NotValidatorOwnerError", CannotStakeError.class);
  mappings.put("TokenNotFound", TokenNotFoundError.class);
  mappings.put("InvalidAccountAddressError", InvalidAccountAddressError.class);
  JSON.registerDiscriminator(InvalidAccountAddressError.class, "type", mappings);
}
}

