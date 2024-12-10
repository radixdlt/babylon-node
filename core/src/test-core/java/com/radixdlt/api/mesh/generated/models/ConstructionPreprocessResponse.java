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
import com.radixdlt.api.mesh.generated.models.AccountIdentifier;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import java.util.ArrayList;
import java.util.List;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * ConstructionPreprocessResponse contains &#x60;options&#x60; that will be sent unmodified to &#x60;/construction/metadata&#x60;. If it is not necessary to make a request to &#x60;/construction/metadata&#x60;, &#x60;options&#x60; should be omitted.   Some blockchains require the PublicKey of particular AccountIdentifiers to construct a valid transaction. To fetch these PublicKeys, populate &#x60;required_public_keys&#x60; with the AccountIdentifiers associated with the desired PublicKeys. If it is not necessary to retrieve any PublicKeys for construction, &#x60;required_public_keys&#x60; should be omitted. 
 */
@ApiModel(description = "ConstructionPreprocessResponse contains `options` that will be sent unmodified to `/construction/metadata`. If it is not necessary to make a request to `/construction/metadata`, `options` should be omitted.   Some blockchains require the PublicKey of particular AccountIdentifiers to construct a valid transaction. To fetch these PublicKeys, populate `required_public_keys` with the AccountIdentifiers associated with the desired PublicKeys. If it is not necessary to retrieve any PublicKeys for construction, `required_public_keys` should be omitted. ")
@JsonPropertyOrder({
  ConstructionPreprocessResponse.JSON_PROPERTY_OPTIONS,
  ConstructionPreprocessResponse.JSON_PROPERTY_REQUIRED_PUBLIC_KEYS
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class ConstructionPreprocessResponse {
  public static final String JSON_PROPERTY_OPTIONS = "options";
  private Object options;

  public static final String JSON_PROPERTY_REQUIRED_PUBLIC_KEYS = "required_public_keys";
  private List<AccountIdentifier> requiredPublicKeys = null;

  public ConstructionPreprocessResponse() { 
  }

  public ConstructionPreprocessResponse options(Object options) {
    this.options = options;
    return this;
  }

   /**
   * The options that will be sent directly to &#x60;/construction/metadata&#x60; by the caller. 
   * @return options
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "The options that will be sent directly to `/construction/metadata` by the caller. ")
  @JsonProperty(JSON_PROPERTY_OPTIONS)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public Object getOptions() {
    return options;
  }


  @JsonProperty(JSON_PROPERTY_OPTIONS)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setOptions(Object options) {
    this.options = options;
  }


  public ConstructionPreprocessResponse requiredPublicKeys(List<AccountIdentifier> requiredPublicKeys) {
    this.requiredPublicKeys = requiredPublicKeys;
    return this;
  }

  public ConstructionPreprocessResponse addRequiredPublicKeysItem(AccountIdentifier requiredPublicKeysItem) {
    if (this.requiredPublicKeys == null) {
      this.requiredPublicKeys = new ArrayList<>();
    }
    this.requiredPublicKeys.add(requiredPublicKeysItem);
    return this;
  }

   /**
   * Get requiredPublicKeys
   * @return requiredPublicKeys
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "")
  @JsonProperty(JSON_PROPERTY_REQUIRED_PUBLIC_KEYS)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public List<AccountIdentifier> getRequiredPublicKeys() {
    return requiredPublicKeys;
  }


  @JsonProperty(JSON_PROPERTY_REQUIRED_PUBLIC_KEYS)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setRequiredPublicKeys(List<AccountIdentifier> requiredPublicKeys) {
    this.requiredPublicKeys = requiredPublicKeys;
  }


  /**
   * Return true if this ConstructionPreprocessResponse object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    ConstructionPreprocessResponse constructionPreprocessResponse = (ConstructionPreprocessResponse) o;
    return Objects.equals(this.options, constructionPreprocessResponse.options) &&
        Objects.equals(this.requiredPublicKeys, constructionPreprocessResponse.requiredPublicKeys);
  }

  @Override
  public int hashCode() {
    return Objects.hash(options, requiredPublicKeys);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class ConstructionPreprocessResponse {\n");
    sb.append("    options: ").append(toIndentedString(options)).append("\n");
    sb.append("    requiredPublicKeys: ").append(toIndentedString(requiredPublicKeys)).append("\n");
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
