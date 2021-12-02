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
 * TokenNotFoundError
 */
@JsonPropertyOrder({
  TokenNotFoundError.JSON_PROPERTY_TOKEN_NOT_FOUND
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen", date = "2021-12-01T23:01:03.351839-06:00[America/Chicago]")
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
  @JsonSubTypes.Type(value = TokenNotFoundError.class, name = "TokenNotFoundError"),
})

public class TokenNotFoundError extends Error {
  public static final String JSON_PROPERTY_TOKEN_NOT_FOUND = "token_not_found";
  private TokenIdentifier tokenNotFound;


  public TokenNotFoundError tokenNotFound(TokenIdentifier tokenNotFound) {
    this.tokenNotFound = tokenNotFound;
    return this;
  }

   /**
   * Get tokenNotFound
   * @return tokenNotFound
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_TOKEN_NOT_FOUND)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public TokenIdentifier getTokenNotFound() {
    return tokenNotFound;
  }


  @JsonProperty(JSON_PROPERTY_TOKEN_NOT_FOUND)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setTokenNotFound(TokenIdentifier tokenNotFound) {
    this.tokenNotFound = tokenNotFound;
  }


  /**
   * Return true if this TokenNotFoundError object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    TokenNotFoundError tokenNotFoundError = (TokenNotFoundError) o;
    return Objects.equals(this.tokenNotFound, tokenNotFoundError.tokenNotFound) &&
        super.equals(o);
  }

  @Override
  public int hashCode() {
    return Objects.hash(tokenNotFound, super.hashCode());
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class TokenNotFoundError {\n");
    sb.append("    ").append(toIndentedString(super.toString())).append("\n");
    sb.append("    tokenNotFound: ").append(toIndentedString(tokenNotFound)).append("\n");
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
  mappings.put("TokenNotFoundError", TokenNotFoundError.class);
  mappings.put("TokenNotFoundError", TokenNotFoundError.class);
  JSON.registerDiscriminator(TokenNotFoundError.class, "type", mappings);
}
}

