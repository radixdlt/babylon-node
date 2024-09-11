/*
 * Radix Core API
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.2.3
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
import com.radixdlt.api.core.generated.models.LedgerProof;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import java.util.ArrayList;
import java.util.List;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * StreamProofsResponse
 */
@JsonPropertyOrder({
  StreamProofsResponse.JSON_PROPERTY_PAGE,
  StreamProofsResponse.JSON_PROPERTY_CONTINUATION_TOKEN
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class StreamProofsResponse {
  public static final String JSON_PROPERTY_PAGE = "page";
  private List<LedgerProof> page = new ArrayList<>();

  public static final String JSON_PROPERTY_CONTINUATION_TOKEN = "continuation_token";
  private String continuationToken;

  public StreamProofsResponse() { 
  }

  public StreamProofsResponse page(List<LedgerProof> page) {
    this.page = page;
    return this;
  }

  public StreamProofsResponse addPageItem(LedgerProof pageItem) {
    this.page.add(pageItem);
    return this;
  }

   /**
   * A page of ledger proofs stored by this node.
   * @return page
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "A page of ledger proofs stored by this node.")
  @JsonProperty(JSON_PROPERTY_PAGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public List<LedgerProof> getPage() {
    return page;
  }


  @JsonProperty(JSON_PROPERTY_PAGE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setPage(List<LedgerProof> page) {
    this.page = page;
  }


  public StreamProofsResponse continuationToken(String continuationToken) {
    this.continuationToken = continuationToken;
    return this;
  }

   /**
   * A continuation token is returned if and only if there are further non-empty pages of items currently available. The token can be provided in a following request to fetch the next page of results. The filter and sort should not be changed when re-using the continuation token. 
   * @return continuationToken
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "A continuation token is returned if and only if there are further non-empty pages of items currently available. The token can be provided in a following request to fetch the next page of results. The filter and sort should not be changed when re-using the continuation token. ")
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
   * Return true if this StreamProofsResponse object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    StreamProofsResponse streamProofsResponse = (StreamProofsResponse) o;
    return Objects.equals(this.page, streamProofsResponse.page) &&
        Objects.equals(this.continuationToken, streamProofsResponse.continuationToken);
  }

  @Override
  public int hashCode() {
    return Objects.hash(page, continuationToken);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class StreamProofsResponse {\n");
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

