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
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * ScryptoInstant
 */
@JsonPropertyOrder({
  ScryptoInstant.JSON_PROPERTY_UNIX_TIMESTAMP_SECONDS,
  ScryptoInstant.JSON_PROPERTY_DATE_TIME
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class ScryptoInstant {
  public static final String JSON_PROPERTY_UNIX_TIMESTAMP_SECONDS = "unix_timestamp_seconds";
  private String unixTimestampSeconds;

  public static final String JSON_PROPERTY_DATE_TIME = "date_time";
  private String dateTime;

  public ScryptoInstant() { 
  }

  public ScryptoInstant unixTimestampSeconds(String unixTimestampSeconds) {
    this.unixTimestampSeconds = unixTimestampSeconds;
    return this;
  }

   /**
   * A decimal string-encoded 64-bit signed integer, marking the unix timestamp in seconds.  Note: this field accurately represents the full range of possible on-ledger values (i.e. &#x60;-2^63 &lt;&#x3D; seconds &lt; 2^63&#x60;). This is contrary to the &#x60;InstantMs&#x60; type used in other places of this API. 
   * @return unixTimestampSeconds
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "A decimal string-encoded 64-bit signed integer, marking the unix timestamp in seconds.  Note: this field accurately represents the full range of possible on-ledger values (i.e. `-2^63 <= seconds < 2^63`). This is contrary to the `InstantMs` type used in other places of this API. ")
  @JsonProperty(JSON_PROPERTY_UNIX_TIMESTAMP_SECONDS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getUnixTimestampSeconds() {
    return unixTimestampSeconds;
  }


  @JsonProperty(JSON_PROPERTY_UNIX_TIMESTAMP_SECONDS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setUnixTimestampSeconds(String unixTimestampSeconds) {
    this.unixTimestampSeconds = unixTimestampSeconds;
  }


  public ScryptoInstant dateTime(String dateTime) {
    this.dateTime = dateTime;
    return this;
  }

   /**
   * The RFC 3339 / ISO 8601 string representation of the timestamp. Will always use \&quot;Z\&quot; (denoting UTC) and a second-precision (i.e. *skipping* the &#x60;.000&#x60; milliseconds part). E.g.: &#x60;2023-01-26T18:30:09Z&#x60;.  Note: This field will *not* be present if the actual on-ledger &#x60;unix_timestamp_seconds&#x60; value is outside the basic range supported by the RFC 3339 / ISO 8601 standard, which starts at year 1583 (i.e. the beginning of the Gregorian calendar) and ends at year 9999 (inclusive). 
   * @return dateTime
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "The RFC 3339 / ISO 8601 string representation of the timestamp. Will always use \"Z\" (denoting UTC) and a second-precision (i.e. *skipping* the `.000` milliseconds part). E.g.: `2023-01-26T18:30:09Z`.  Note: This field will *not* be present if the actual on-ledger `unix_timestamp_seconds` value is outside the basic range supported by the RFC 3339 / ISO 8601 standard, which starts at year 1583 (i.e. the beginning of the Gregorian calendar) and ends at year 9999 (inclusive). ")
  @JsonProperty(JSON_PROPERTY_DATE_TIME)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public String getDateTime() {
    return dateTime;
  }


  @JsonProperty(JSON_PROPERTY_DATE_TIME)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setDateTime(String dateTime) {
    this.dateTime = dateTime;
  }


  /**
   * Return true if this ScryptoInstant object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    ScryptoInstant scryptoInstant = (ScryptoInstant) o;
    return Objects.equals(this.unixTimestampSeconds, scryptoInstant.unixTimestampSeconds) &&
        Objects.equals(this.dateTime, scryptoInstant.dateTime);
  }

  @Override
  public int hashCode() {
    return Objects.hash(unixTimestampSeconds, dateTime);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class ScryptoInstant {\n");
    sb.append("    unixTimestampSeconds: ").append(toIndentedString(unixTimestampSeconds)).append("\n");
    sb.append("    dateTime: ").append(toIndentedString(dateTime)).append("\n");
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

