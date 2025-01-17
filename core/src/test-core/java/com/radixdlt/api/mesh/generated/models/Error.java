/*
 * Rosetta
 * Build Once. Integrate Your Blockchain Everywhere. 
 *
 * The version of the OpenAPI document: 1.4.13
 * 
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */


package com.radixdlt.api.mesh.generated.models;

import java.util.Objects;
import java.util.Arrays;
import java.util.Map;
import java.util.HashMap;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonTypeName;
import com.fasterxml.jackson.annotation.JsonValue;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * Instead of utilizing HTTP status codes to describe node errors (which often do not have a good analog), rich errors are returned using this object.  Both the code and message fields can be individually used to correctly identify an error. Implementations MUST use unique values for both fields. 
 */
@ApiModel(description = "Instead of utilizing HTTP status codes to describe node errors (which often do not have a good analog), rich errors are returned using this object.  Both the code and message fields can be individually used to correctly identify an error. Implementations MUST use unique values for both fields. ")
@JsonPropertyOrder({
  Error.JSON_PROPERTY_CODE,
  Error.JSON_PROPERTY_MESSAGE,
  Error.JSON_PROPERTY_DESCRIPTION,
  Error.JSON_PROPERTY_RETRIABLE,
  Error.JSON_PROPERTY_DETAILS
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class Error {
  public static final String JSON_PROPERTY_CODE = "code";
  private Integer code;

  public static final String JSON_PROPERTY_MESSAGE = "message";
  private String message;

  public static final String JSON_PROPERTY_DESCRIPTION = "description";
  private String description;

  public static final String JSON_PROPERTY_RETRIABLE = "retriable";
  private Boolean retriable;

  public static final String JSON_PROPERTY_DETAILS = "details";
  private Object details;

  public Error() { 
  }

  public Error code(Integer code) {
    this.code = code;
    return this;
  }

   /**
   * Code is a network-specific error code. If desired, this code can be equivalent to an HTTP status code. 
   * minimum: 0
   * @return code
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(example = "12", required = true, value = "Code is a network-specific error code. If desired, this code can be equivalent to an HTTP status code. ")
  @JsonProperty(JSON_PROPERTY_CODE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Integer getCode() {
    return code;
  }


  @JsonProperty(JSON_PROPERTY_CODE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setCode(Integer code) {
    this.code = code;
  }


  public Error message(String message) {
    this.message = message;
    return this;
  }

   /**
   * Message is a network-specific error message.  The message MUST NOT change for a given code. In particular, this means that any contextual information should be included in the details field. 
   * @return message
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(example = "Invalid account format", required = true, value = "Message is a network-specific error message.  The message MUST NOT change for a given code. In particular, this means that any contextual information should be included in the details field. ")
  @JsonProperty(JSON_PROPERTY_MESSAGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getMessage() {
    return message;
  }


  @JsonProperty(JSON_PROPERTY_MESSAGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setMessage(String message) {
    this.message = message;
  }


  public Error description(String description) {
    this.description = description;
    return this;
  }

   /**
   * Description allows the implementer to optionally provide additional information about an error. In many cases, the content of this field will be a copy-and-paste from existing developer documentation.  Description can ONLY be populated with generic information about a particular type of error. It MUST NOT be populated with information about a particular instantiation of an error (use &#x60;details&#x60; for this).  Whereas the content of Error.Message should stay stable across releases, the content of Error.Description will likely change across releases (as implementers improve error documentation). For this reason, the content in this field is not part of any type assertion (unlike Error.Message). 
   * @return description
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(example = "This error is returned when the requested AccountIdentifier is improperly formatted.", value = "Description allows the implementer to optionally provide additional information about an error. In many cases, the content of this field will be a copy-and-paste from existing developer documentation.  Description can ONLY be populated with generic information about a particular type of error. It MUST NOT be populated with information about a particular instantiation of an error (use `details` for this).  Whereas the content of Error.Message should stay stable across releases, the content of Error.Description will likely change across releases (as implementers improve error documentation). For this reason, the content in this field is not part of any type assertion (unlike Error.Message). ")
  @JsonProperty(JSON_PROPERTY_DESCRIPTION)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public String getDescription() {
    return description;
  }


  @JsonProperty(JSON_PROPERTY_DESCRIPTION)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setDescription(String description) {
    this.description = description;
  }


  public Error retriable(Boolean retriable) {
    this.retriable = retriable;
    return this;
  }

   /**
   * An error is retriable if the same request may succeed if submitted again. 
   * @return retriable
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "An error is retriable if the same request may succeed if submitted again. ")
  @JsonProperty(JSON_PROPERTY_RETRIABLE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Boolean getRetriable() {
    return retriable;
  }


  @JsonProperty(JSON_PROPERTY_RETRIABLE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setRetriable(Boolean retriable) {
    this.retriable = retriable;
  }


  public Error details(Object details) {
    this.details = details;
    return this;
  }

   /**
   * Often times it is useful to return context specific to the request that caused the error (i.e. a sample of the stack trace or impacted account) in addition to the standard error message. 
   * @return details
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(example = "{address=0x1dcc4de8dec75d7aab85b567b6, error=not base64}", value = "Often times it is useful to return context specific to the request that caused the error (i.e. a sample of the stack trace or impacted account) in addition to the standard error message. ")
  @JsonProperty(JSON_PROPERTY_DETAILS)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public Object getDetails() {
    return details;
  }


  @JsonProperty(JSON_PROPERTY_DETAILS)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setDetails(Object details) {
    this.details = details;
  }


  /**
   * Return true if this Error object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    Error error = (Error) o;
    return Objects.equals(this.code, error.code) &&
        Objects.equals(this.message, error.message) &&
        Objects.equals(this.description, error.description) &&
        Objects.equals(this.retriable, error.retriable) &&
        Objects.equals(this.details, error.details);
  }

  @Override
  public int hashCode() {
    return Objects.hash(code, message, description, retriable, details);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class Error {\n");
    sb.append("    code: ").append(toIndentedString(code)).append("\n");
    sb.append("    message: ").append(toIndentedString(message)).append("\n");
    sb.append("    description: ").append(toIndentedString(description)).append("\n");
    sb.append("    retriable: ").append(toIndentedString(retriable)).append("\n");
    sb.append("    details: ").append(toIndentedString(details)).append("\n");
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

