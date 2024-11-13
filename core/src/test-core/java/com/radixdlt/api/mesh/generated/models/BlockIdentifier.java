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
 * The block_identifier uniquely identifies a block in a particular network. 
 */
@ApiModel(description = "The block_identifier uniquely identifies a block in a particular network. ")
@JsonPropertyOrder({
  BlockIdentifier.JSON_PROPERTY_INDEX,
  BlockIdentifier.JSON_PROPERTY_HASH
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class BlockIdentifier {
  public static final String JSON_PROPERTY_INDEX = "index";
  private Long index;

  public static final String JSON_PROPERTY_HASH = "hash";
  private String hash;

  public BlockIdentifier() { 
  }

  public BlockIdentifier index(Long index) {
    this.index = index;
    return this;
  }

   /**
   * This is also known as the block height. 
   * @return index
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(example = "1123941", required = true, value = "This is also known as the block height. ")
  @JsonProperty(JSON_PROPERTY_INDEX)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Long getIndex() {
    return index;
  }


  @JsonProperty(JSON_PROPERTY_INDEX)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setIndex(Long index) {
    this.index = index;
  }


  public BlockIdentifier hash(String hash) {
    this.hash = hash;
    return this;
  }

   /**
   * This should be normalized according to the case specified in the block_hash_case network options. 
   * @return hash
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(example = "0x1f2cc6c5027d2f201a5453ad1119574d2aed23a392654742ac3c78783c071f85", required = true, value = "This should be normalized according to the case specified in the block_hash_case network options. ")
  @JsonProperty(JSON_PROPERTY_HASH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public String getHash() {
    return hash;
  }


  @JsonProperty(JSON_PROPERTY_HASH)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setHash(String hash) {
    this.hash = hash;
  }


  /**
   * Return true if this BlockIdentifier object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    BlockIdentifier blockIdentifier = (BlockIdentifier) o;
    return Objects.equals(this.index, blockIdentifier.index) &&
        Objects.equals(this.hash, blockIdentifier.hash);
  }

  @Override
  public int hashCode() {
    return Objects.hash(index, hash);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class BlockIdentifier {\n");
    sb.append("    index: ").append(toIndentedString(index)).append("\n");
    sb.append("    hash: ").append(toIndentedString(hash)).append("\n");
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
