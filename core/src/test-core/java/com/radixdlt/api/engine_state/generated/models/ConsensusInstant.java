/*
 * Engine State API (Beta)
 * **This API is currently in Beta**  This specification may experience breaking changes as part of Babylon Node releases. Such changes will be clearly mentioned in the [babylon-node release notes](https://github.com/radixdlt/babylon-node/releases). We advise against using this API for business-critical integrations before the `version` indicated above becomes stable, which is expected in Q4 of 2024.  This API provides a complete view of the current ledger state, operating at a relatively low level (i.e. returning Entities' data and type information in a generic way, without interpreting specifics of different native or custom components).  It mirrors how the Radix Engine views the ledger state in its \"System\" layer, and thus can be useful for Scrypto developers, who need to inspect how the Engine models and stores their application's state, or how an interface / authentication scheme of another component looks like. 
 *
 * The version of the OpenAPI document: v1.2.3
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
 * ConsensusInstant
 */
@JsonPropertyOrder({
  ConsensusInstant.JSON_PROPERTY_UNIX_TIMESTAMP_MS,
  ConsensusInstant.JSON_PROPERTY_DATE_TIME
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class ConsensusInstant {
  public static final String JSON_PROPERTY_UNIX_TIMESTAMP_MS = "unix_timestamp_ms";
  private String unixTimestampMs;

  public static final String JSON_PROPERTY_DATE_TIME = "date_time";
  private String dateTime;

  public ConsensusInstant() { 
  }

  public ConsensusInstant unixTimestampMs(String unixTimestampMs) {
    this.unixTimestampMs = unixTimestampMs;
    return this;
  }

   /**
   * A decimal string-encoded 64-bit signed integer, marking the unix timestamp in milliseconds.  Note: this field accurately represents the full range of possible values (i.e. &#x60;-2^63 &lt;&#x3D; milliseconds &lt; 2^63&#x60;). 
   * @return unixTimestampMs
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "A decimal string-encoded 64-bit signed integer, marking the unix timestamp in milliseconds.  Note: this field accurately represents the full range of possible values (i.e. `-2^63 <= milliseconds < 2^63`). ")
  @JsonProperty(JSON_PROPERTY_UNIX_TIMESTAMP_MS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getUnixTimestampMs() {
    return unixTimestampMs;
  }


  @JsonProperty(JSON_PROPERTY_UNIX_TIMESTAMP_MS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setUnixTimestampMs(String unixTimestampMs) {
    this.unixTimestampMs = unixTimestampMs;
  }


  public ConsensusInstant dateTime(String dateTime) {
    this.dateTime = dateTime;
    return this;
  }

   /**
   * The RFC 3339 / ISO 8601 string representation of the timestamp. Will always use \&quot;Z\&quot; (denoting UTC) and include milliseconds (even if &#x60;000&#x60;). E.g.: &#x60;2023-01-26T18:30:09.453Z&#x60;.  Note: This field will *not* be present if the &#x60;unix_timestamp_ms&#x60; value is outside the basic range supported by the RFC 3339 / ISO 8601 standard, which starts at year 1583 (i.e. the beginning of the Gregorian calendar) and ends at year 9999 (inclusive). 
   * @return dateTime
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "The RFC 3339 / ISO 8601 string representation of the timestamp. Will always use \"Z\" (denoting UTC) and include milliseconds (even if `000`). E.g.: `2023-01-26T18:30:09.453Z`.  Note: This field will *not* be present if the `unix_timestamp_ms` value is outside the basic range supported by the RFC 3339 / ISO 8601 standard, which starts at year 1583 (i.e. the beginning of the Gregorian calendar) and ends at year 9999 (inclusive). ")
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
   * Return true if this ConsensusInstant object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    ConsensusInstant consensusInstant = (ConsensusInstant) o;
    return Objects.equals(this.unixTimestampMs, consensusInstant.unixTimestampMs) &&
        Objects.equals(this.dateTime, consensusInstant.dateTime);
  }

  @Override
  public int hashCode() {
    return Objects.hash(unixTimestampMs, dateTime);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class ConsensusInstant {\n");
    sb.append("    unixTimestampMs: ").append(toIndentedString(unixTimestampMs)).append("\n");
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

