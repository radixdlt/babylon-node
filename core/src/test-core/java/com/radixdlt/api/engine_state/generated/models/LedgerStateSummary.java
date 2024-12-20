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
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonTypeName;
import com.fasterxml.jackson.annotation.JsonValue;
import com.radixdlt.api.engine_state.generated.models.LedgerHeaderSummary;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * A state version and summarized header of the ledger proof which can be used to verify the returned on-ledger data.  Please note that: - For \&quot;current top-of-ledger\&quot; requests (i.e. not specifying any &#x60;LedgerStateSelector&#x60;),   this will always be the most recent ledger header, proving exactly the version at which   the on-ledger data was read. - For historical requests (i.e. using a &#x60;LedgerStateSelector&#x60;), this will be the *nearest*   ledger header at *or after* the requested past state version - depending on the   granularity of the consensus progress (and the granularity of the ledger proofs actually   persisted by the queried Node). 
 */
@ApiModel(description = "A state version and summarized header of the ledger proof which can be used to verify the returned on-ledger data.  Please note that: - For \"current top-of-ledger\" requests (i.e. not specifying any `LedgerStateSelector`),   this will always be the most recent ledger header, proving exactly the version at which   the on-ledger data was read. - For historical requests (i.e. using a `LedgerStateSelector`), this will be the *nearest*   ledger header at *or after* the requested past state version - depending on the   granularity of the consensus progress (and the granularity of the ledger proofs actually   persisted by the queried Node). ")
@JsonPropertyOrder({
  LedgerStateSummary.JSON_PROPERTY_STATE_VERSION,
  LedgerStateSummary.JSON_PROPERTY_HEADER_SUMMARY
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class LedgerStateSummary {
  public static final String JSON_PROPERTY_STATE_VERSION = "state_version";
  private Long stateVersion;

  public static final String JSON_PROPERTY_HEADER_SUMMARY = "header_summary";
  private LedgerHeaderSummary headerSummary;

  public LedgerStateSummary() { 
  }

  public LedgerStateSummary stateVersion(Long stateVersion) {
    this.stateVersion = stateVersion;
    return this;
  }

   /**
   * Get stateVersion
   * minimum: 1
   * maximum: 100000000000000
   * @return stateVersion
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_STATE_VERSION)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Long getStateVersion() {
    return stateVersion;
  }


  @JsonProperty(JSON_PROPERTY_STATE_VERSION)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setStateVersion(Long stateVersion) {
    this.stateVersion = stateVersion;
  }


  public LedgerStateSummary headerSummary(LedgerHeaderSummary headerSummary) {
    this.headerSummary = headerSummary;
    return this;
  }

   /**
   * Get headerSummary
   * @return headerSummary
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_HEADER_SUMMARY)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public LedgerHeaderSummary getHeaderSummary() {
    return headerSummary;
  }


  @JsonProperty(JSON_PROPERTY_HEADER_SUMMARY)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setHeaderSummary(LedgerHeaderSummary headerSummary) {
    this.headerSummary = headerSummary;
  }


  /**
   * Return true if this LedgerStateSummary object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    LedgerStateSummary ledgerStateSummary = (LedgerStateSummary) o;
    return Objects.equals(this.stateVersion, ledgerStateSummary.stateVersion) &&
        Objects.equals(this.headerSummary, ledgerStateSummary.headerSummary);
  }

  @Override
  public int hashCode() {
    return Objects.hash(stateVersion, headerSummary);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class LedgerStateSummary {\n");
    sb.append("    stateVersion: ").append(toIndentedString(stateVersion)).append("\n");
    sb.append("    headerSummary: ").append(toIndentedString(headerSummary)).append("\n");
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

