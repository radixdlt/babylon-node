/*
 * Radix Core API
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.2.2
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
import com.radixdlt.api.core.generated.models.SborData;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * LocalTypeId
 */
@JsonPropertyOrder({
  LocalTypeId.JSON_PROPERTY_KIND,
  LocalTypeId.JSON_PROPERTY_ID,
  LocalTypeId.JSON_PROPERTY_AS_SBOR
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class LocalTypeId {
  /**
   * The location against which to resolve this type reference.
   */
  public enum KindEnum {
    WELLKNOWN("WellKnown"),
    
    SCHEMALOCAL("SchemaLocal");

    private String value;

    KindEnum(String value) {
      this.value = value;
    }

    @JsonValue
    public String getValue() {
      return value;
    }

    @Override
    public String toString() {
      return String.valueOf(value);
    }

    @JsonCreator
    public static KindEnum fromValue(String value) {
      for (KindEnum b : KindEnum.values()) {
        if (b.value.equals(value)) {
          return b;
        }
      }
      throw new IllegalArgumentException("Unexpected value '" + value + "'");
    }
  }

  public static final String JSON_PROPERTY_KIND = "kind";
  private KindEnum kind;

  public static final String JSON_PROPERTY_ID = "id";
  private Long id;

  public static final String JSON_PROPERTY_AS_SBOR = "as_sbor";
  private SborData asSbor;

  public LocalTypeId() { 
  }

  public LocalTypeId kind(KindEnum kind) {
    this.kind = kind;
    return this;
  }

   /**
   * The location against which to resolve this type reference.
   * @return kind
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "The location against which to resolve this type reference.")
  @JsonProperty(JSON_PROPERTY_KIND)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public KindEnum getKind() {
    return kind;
  }


  @JsonProperty(JSON_PROPERTY_KIND)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setKind(KindEnum kind) {
    this.kind = kind;
  }


  public LocalTypeId id(Long id) {
    this.id = id;
    return this;
  }

   /**
   * A reference to a type, interpreted according to &#x60;kind&#x60;: - If &#x60;WellKnown&#x60;, then it is a pointer to a well known scrypto type with that ID, - If &#x60;SchemaLocal&#x60;, then it is an index into the given schema. 
   * minimum: 0
   * maximum: 4294967295
   * @return id
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "A reference to a type, interpreted according to `kind`: - If `WellKnown`, then it is a pointer to a well known scrypto type with that ID, - If `SchemaLocal`, then it is an index into the given schema. ")
  @JsonProperty(JSON_PROPERTY_ID)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Long getId() {
    return id;
  }


  @JsonProperty(JSON_PROPERTY_ID)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setId(Long id) {
    this.id = id;
  }


  public LocalTypeId asSbor(SborData asSbor) {
    this.asSbor = asSbor;
    return this;
  }

   /**
   * Get asSbor
   * @return asSbor
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_AS_SBOR)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public SborData getAsSbor() {
    return asSbor;
  }


  @JsonProperty(JSON_PROPERTY_AS_SBOR)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setAsSbor(SborData asSbor) {
    this.asSbor = asSbor;
  }


  /**
   * Return true if this LocalTypeId object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    LocalTypeId localTypeId = (LocalTypeId) o;
    return Objects.equals(this.kind, localTypeId.kind) &&
        Objects.equals(this.id, localTypeId.id) &&
        Objects.equals(this.asSbor, localTypeId.asSbor);
  }

  @Override
  public int hashCode() {
    return Objects.hash(kind, id, asSbor);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class LocalTypeId {\n");
    sb.append("    kind: ").append(toIndentedString(kind)).append("\n");
    sb.append("    id: ").append(toIndentedString(id)).append("\n");
    sb.append("    asSbor: ").append(toIndentedString(asSbor)).append("\n");
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

