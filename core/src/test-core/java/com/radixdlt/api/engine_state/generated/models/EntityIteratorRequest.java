/*
 * Engine State API
 * This API provides a complete view of the current ledger state, operating at a relatively low level (i.e. returning Entities' data and type information in a generic way, without interpreting specifics of different native or custom components).  It mirrors how the Radix Engine views the ledger state in its \"System\" layer, and thus can be useful for Scrypto developers, who need to inspect how the Engine models and stores their application's state, or how an interface / authentication scheme of another component looks like. 
 *
 * The version of the OpenAPI document: v0.0.1
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
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonTypeName;
import com.fasterxml.jackson.annotation.JsonValue;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * EntityIteratorRequest
 */
@JsonPropertyOrder({
  EntityIteratorRequest.JSON_PROPERTY_MAX_PAGE_SIZE,
  EntityIteratorRequest.JSON_PROPERTY_CONTINUATION_TOKEN
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class EntityIteratorRequest {
  public static final String JSON_PROPERTY_MAX_PAGE_SIZE = "max_page_size";
  private Integer maxPageSize;

  public static final String JSON_PROPERTY_CONTINUATION_TOKEN = "continuation_token";
  private String continuationToken;

  public EntityIteratorRequest() { 
  }

  public EntityIteratorRequest maxPageSize(Integer maxPageSize) {
    this.maxPageSize = maxPageSize;
    return this;
  }

   /**
   * A maximum number of items to be included in the paged listing response. By default, each paged listing endpoint imposes its own limit on the number of returned items (which may even be driven dynamically by system load, etc). This client-provided maximum page size simply adds a further constraint (i.e. can only lower down the number of returned items). 
   * minimum: 1
   * maximum: 1000
   * @return maxPageSize
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "A maximum number of items to be included in the paged listing response. By default, each paged listing endpoint imposes its own limit on the number of returned items (which may even be driven dynamically by system load, etc). This client-provided maximum page size simply adds a further constraint (i.e. can only lower down the number of returned items). ")
  @JsonProperty(JSON_PROPERTY_MAX_PAGE_SIZE)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public Integer getMaxPageSize() {
    return maxPageSize;
  }


  @JsonProperty(JSON_PROPERTY_MAX_PAGE_SIZE)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setMaxPageSize(Integer maxPageSize) {
    this.maxPageSize = maxPageSize;
  }


  public EntityIteratorRequest continuationToken(String continuationToken) {
    this.continuationToken = continuationToken;
    return this;
  }

   /**
   * An opaque string conveying the information on where the next page of results starts. It is returned in every paged listing response (except for the last page), and it can be passed in every paged listing request (in order to begin listing from where the previous response ended). 
   * @return continuationToken
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "An opaque string conveying the information on where the next page of results starts. It is returned in every paged listing response (except for the last page), and it can be passed in every paged listing request (in order to begin listing from where the previous response ended). ")
  @JsonProperty(JSON_PROPERTY_CONTINUATION_TOKEN)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public String getContinuationToken() {
    return continuationToken;
  }


  @JsonProperty(JSON_PROPERTY_CONTINUATION_TOKEN)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setContinuationToken(String continuationToken) {
    this.continuationToken = continuationToken;
  }


  /**
   * Return true if this EntityIteratorRequest object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    EntityIteratorRequest entityIteratorRequest = (EntityIteratorRequest) o;
    return Objects.equals(this.maxPageSize, entityIteratorRequest.maxPageSize) &&
        Objects.equals(this.continuationToken, entityIteratorRequest.continuationToken);
  }

  @Override
  public int hashCode() {
    return Objects.hash(maxPageSize, continuationToken);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class EntityIteratorRequest {\n");
    sb.append("    maxPageSize: ").append(toIndentedString(maxPageSize)).append("\n");
    sb.append("    continuationToken: ").append(toIndentedString(continuationToken)).append("\n");
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
