/*
 * Engine State API - Babylon (Anemone)
 * **This API is currently in Beta**  This specification may experience breaking changes as part of Babylon Node releases. Such changes will be clearly mentioned in the [babylon-node release notes](https://github.com/radixdlt/babylon-node/releases). We advise against using this API for business-critical integrations before the `version` indicated above becomes stable, which is expected in Q4 of 2024.  This API provides a complete view of the current ledger state, operating at a relatively low level (i.e. returning Entities' data and type information in a generic way, without interpreting specifics of different native or custom components).  It mirrors how the Radix Engine views the ledger state in its \"System\" layer, and thus can be useful for Scrypto developers, who need to inspect how the Engine models and stores their application's state, or how an interface / authentication scheme of another component looks like. 
 *
 * The version of the OpenAPI document: v0.1-beta
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
import com.radixdlt.api.engine_state.generated.models.LedgerStateSummary;
import com.radixdlt.api.engine_state.generated.models.ListedEntityItem;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import java.util.ArrayList;
import java.util.List;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * ExtraEntitySearchResponse
 */
@JsonPropertyOrder({
  ExtraEntitySearchResponse.JSON_PROPERTY_AT_LEDGER_STATE,
  ExtraEntitySearchResponse.JSON_PROPERTY_PAGE,
  ExtraEntitySearchResponse.JSON_PROPERTY_CONTINUATION_TOKEN
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class ExtraEntitySearchResponse {
  public static final String JSON_PROPERTY_AT_LEDGER_STATE = "at_ledger_state";
  private LedgerStateSummary atLedgerState;

  public static final String JSON_PROPERTY_PAGE = "page";
  private List<ListedEntityItem> page = new ArrayList<>();

  public static final String JSON_PROPERTY_CONTINUATION_TOKEN = "continuation_token";
  private String continuationToken;

  public ExtraEntitySearchResponse() { 
  }

  public ExtraEntitySearchResponse atLedgerState(LedgerStateSummary atLedgerState) {
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


  public ExtraEntitySearchResponse page(List<ListedEntityItem> page) {
    this.page = page;
    return this;
  }

  public ExtraEntitySearchResponse addPageItem(ListedEntityItem pageItem) {
    this.page.add(pageItem);
    return this;
  }

   /**
   * A page of entities. If this page is the last one, the &#x60;continuation_token&#x60; will be not be included.
   * @return page
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "A page of entities. If this page is the last one, the `continuation_token` will be not be included.")
  @JsonProperty(JSON_PROPERTY_PAGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public List<ListedEntityItem> getPage() {
    return page;
  }


  @JsonProperty(JSON_PROPERTY_PAGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setPage(List<ListedEntityItem> page) {
    this.page = page;
  }


  public ExtraEntitySearchResponse continuationToken(String continuationToken) {
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
   * Return true if this ExtraEntitySearchResponse object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    ExtraEntitySearchResponse extraEntitySearchResponse = (ExtraEntitySearchResponse) o;
    return Objects.equals(this.atLedgerState, extraEntitySearchResponse.atLedgerState) &&
        Objects.equals(this.page, extraEntitySearchResponse.page) &&
        Objects.equals(this.continuationToken, extraEntitySearchResponse.continuationToken);
  }

  @Override
  public int hashCode() {
    return Objects.hash(atLedgerState, page, continuationToken);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class ExtraEntitySearchResponse {\n");
    sb.append("    atLedgerState: ").append(toIndentedString(atLedgerState)).append("\n");
    sb.append("    page: ").append(toIndentedString(page)).append("\n");
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

