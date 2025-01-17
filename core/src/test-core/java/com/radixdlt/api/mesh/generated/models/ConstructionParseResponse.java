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
import com.radixdlt.api.mesh.generated.models.Operation;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import java.util.ArrayList;
import java.util.List;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * ConstructionParseResponse contains an array of operations that occur in a transaction blob. This should match the array of operations provided to &#x60;/construction/preprocess&#x60; and &#x60;/construction/payloads&#x60;. 
 */
@ApiModel(description = "ConstructionParseResponse contains an array of operations that occur in a transaction blob. This should match the array of operations provided to `/construction/preprocess` and `/construction/payloads`. ")
@JsonPropertyOrder({
  ConstructionParseResponse.JSON_PROPERTY_OPERATIONS,
  ConstructionParseResponse.JSON_PROPERTY_SIGNERS,
  ConstructionParseResponse.JSON_PROPERTY_ACCOUNT_IDENTIFIER_SIGNERS,
  ConstructionParseResponse.JSON_PROPERTY_METADATA
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class ConstructionParseResponse {
  public static final String JSON_PROPERTY_OPERATIONS = "operations";
  private List<Operation> operations = new ArrayList<>();

  public static final String JSON_PROPERTY_SIGNERS = "signers";
  private List<String> signers = null;

  public static final String JSON_PROPERTY_ACCOUNT_IDENTIFIER_SIGNERS = "account_identifier_signers";
  private List<AccountIdentifier> accountIdentifierSigners = null;

  public static final String JSON_PROPERTY_METADATA = "metadata";
  private Object metadata;

  public ConstructionParseResponse() { 
  }

  public ConstructionParseResponse operations(List<Operation> operations) {
    this.operations = operations;
    return this;
  }

  public ConstructionParseResponse addOperationsItem(Operation operationsItem) {
    this.operations.add(operationsItem);
    return this;
  }

   /**
   * Get operations
   * @return operations
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_OPERATIONS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public List<Operation> getOperations() {
    return operations;
  }


  @JsonProperty(JSON_PROPERTY_OPERATIONS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setOperations(List<Operation> operations) {
    this.operations = operations;
  }


  public ConstructionParseResponse signers(List<String> signers) {
    this.signers = signers;
    return this;
  }

  public ConstructionParseResponse addSignersItem(String signersItem) {
    if (this.signers == null) {
      this.signers = new ArrayList<>();
    }
    this.signers.add(signersItem);
    return this;
  }

   /**
   * [DEPRECATED by &#x60;account_identifier_signers&#x60; in &#x60;v1.4.4&#x60;] All signers (addresses) of a particular transaction. If the transaction is unsigned, it should be empty. 
   * @return signers
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "[DEPRECATED by `account_identifier_signers` in `v1.4.4`] All signers (addresses) of a particular transaction. If the transaction is unsigned, it should be empty. ")
  @JsonProperty(JSON_PROPERTY_SIGNERS)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public List<String> getSigners() {
    return signers;
  }


  @JsonProperty(JSON_PROPERTY_SIGNERS)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setSigners(List<String> signers) {
    this.signers = signers;
  }


  public ConstructionParseResponse accountIdentifierSigners(List<AccountIdentifier> accountIdentifierSigners) {
    this.accountIdentifierSigners = accountIdentifierSigners;
    return this;
  }

  public ConstructionParseResponse addAccountIdentifierSignersItem(AccountIdentifier accountIdentifierSignersItem) {
    if (this.accountIdentifierSigners == null) {
      this.accountIdentifierSigners = new ArrayList<>();
    }
    this.accountIdentifierSigners.add(accountIdentifierSignersItem);
    return this;
  }

   /**
   * Get accountIdentifierSigners
   * @return accountIdentifierSigners
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "")
  @JsonProperty(JSON_PROPERTY_ACCOUNT_IDENTIFIER_SIGNERS)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public List<AccountIdentifier> getAccountIdentifierSigners() {
    return accountIdentifierSigners;
  }


  @JsonProperty(JSON_PROPERTY_ACCOUNT_IDENTIFIER_SIGNERS)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setAccountIdentifierSigners(List<AccountIdentifier> accountIdentifierSigners) {
    this.accountIdentifierSigners = accountIdentifierSigners;
  }


  public ConstructionParseResponse metadata(Object metadata) {
    this.metadata = metadata;
    return this;
  }

   /**
   * Get metadata
   * @return metadata
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "")
  @JsonProperty(JSON_PROPERTY_METADATA)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public Object getMetadata() {
    return metadata;
  }


  @JsonProperty(JSON_PROPERTY_METADATA)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setMetadata(Object metadata) {
    this.metadata = metadata;
  }


  /**
   * Return true if this ConstructionParseResponse object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    ConstructionParseResponse constructionParseResponse = (ConstructionParseResponse) o;
    return Objects.equals(this.operations, constructionParseResponse.operations) &&
        Objects.equals(this.signers, constructionParseResponse.signers) &&
        Objects.equals(this.accountIdentifierSigners, constructionParseResponse.accountIdentifierSigners) &&
        Objects.equals(this.metadata, constructionParseResponse.metadata);
  }

  @Override
  public int hashCode() {
    return Objects.hash(operations, signers, accountIdentifierSigners, metadata);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class ConstructionParseResponse {\n");
    sb.append("    operations: ").append(toIndentedString(operations)).append("\n");
    sb.append("    signers: ").append(toIndentedString(signers)).append("\n");
    sb.append("    accountIdentifierSigners: ").append(toIndentedString(accountIdentifierSigners)).append("\n");
    sb.append("    metadata: ").append(toIndentedString(metadata)).append("\n");
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
