/*
 * Radix Core API - Babylon
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.0.0
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
import com.radixdlt.api.core.generated.models.LedgerHeader;
import com.radixdlt.api.core.generated.models.TimestampedValidatorSignature;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import java.util.ArrayList;
import java.util.List;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * LedgerProof
 */
@JsonPropertyOrder({
  LedgerProof.JSON_PROPERTY_OPAQUE_HASH,
  LedgerProof.JSON_PROPERTY_LEDGER_HEADER,
  LedgerProof.JSON_PROPERTY_TIMESTAMPED_SIGNATURES
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class LedgerProof {
  public static final String JSON_PROPERTY_OPAQUE_HASH = "opaque_hash";
  private String opaqueHash;

  public static final String JSON_PROPERTY_LEDGER_HEADER = "ledger_header";
  private LedgerHeader ledgerHeader;

  public static final String JSON_PROPERTY_TIMESTAMPED_SIGNATURES = "timestamped_signatures";
  private List<TimestampedValidatorSignature> timestampedSignatures = new ArrayList<>();

  public LedgerProof() { 
  }

  public LedgerProof opaqueHash(String opaqueHash) {
    this.opaqueHash = opaqueHash;
    return this;
  }

   /**
   * A hex-encoded 32-byte vertex VoteData hash on the consensus side, opaque to ledger.
   * @return opaqueHash
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "A hex-encoded 32-byte vertex VoteData hash on the consensus side, opaque to ledger.")
  @JsonProperty(JSON_PROPERTY_OPAQUE_HASH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getOpaqueHash() {
    return opaqueHash;
  }


  @JsonProperty(JSON_PROPERTY_OPAQUE_HASH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setOpaqueHash(String opaqueHash) {
    this.opaqueHash = opaqueHash;
  }


  public LedgerProof ledgerHeader(LedgerHeader ledgerHeader) {
    this.ledgerHeader = ledgerHeader;
    return this;
  }

   /**
   * Get ledgerHeader
   * @return ledgerHeader
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_LEDGER_HEADER)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public LedgerHeader getLedgerHeader() {
    return ledgerHeader;
  }


  @JsonProperty(JSON_PROPERTY_LEDGER_HEADER)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setLedgerHeader(LedgerHeader ledgerHeader) {
    this.ledgerHeader = ledgerHeader;
  }


  public LedgerProof timestampedSignatures(List<TimestampedValidatorSignature> timestampedSignatures) {
    this.timestampedSignatures = timestampedSignatures;
    return this;
  }

  public LedgerProof addTimestampedSignaturesItem(TimestampedValidatorSignature timestampedSignaturesItem) {
    this.timestampedSignatures.add(timestampedSignaturesItem);
    return this;
  }

   /**
   * Get timestampedSignatures
   * @return timestampedSignatures
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_TIMESTAMPED_SIGNATURES)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public List<TimestampedValidatorSignature> getTimestampedSignatures() {
    return timestampedSignatures;
  }


  @JsonProperty(JSON_PROPERTY_TIMESTAMPED_SIGNATURES)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setTimestampedSignatures(List<TimestampedValidatorSignature> timestampedSignatures) {
    this.timestampedSignatures = timestampedSignatures;
  }


  /**
   * Return true if this LedgerProof object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    LedgerProof ledgerProof = (LedgerProof) o;
    return Objects.equals(this.opaqueHash, ledgerProof.opaqueHash) &&
        Objects.equals(this.ledgerHeader, ledgerProof.ledgerHeader) &&
        Objects.equals(this.timestampedSignatures, ledgerProof.timestampedSignatures);
  }

  @Override
  public int hashCode() {
    return Objects.hash(opaqueHash, ledgerHeader, timestampedSignatures);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class LedgerProof {\n");
    sb.append("    opaqueHash: ").append(toIndentedString(opaqueHash)).append("\n");
    sb.append("    ledgerHeader: ").append(toIndentedString(ledgerHeader)).append("\n");
    sb.append("    timestampedSignatures: ").append(toIndentedString(timestampedSignatures)).append("\n");
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

