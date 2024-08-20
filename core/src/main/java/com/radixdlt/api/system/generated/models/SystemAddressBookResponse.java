/*
 * Radix System API
 * This API is exposed by the Babylon Radix node to give clients access to information about the node itself, its configuration, status and subsystems.  It is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Heavy load may impact the node's function.  If you require queries against ledger state, you may also wish to consider using the [Core API or Gateway API instead](https://docs-babylon.radixdlt.com/main/apis/api-specification.html). 
 *
 * The version of the OpenAPI document: v1.2.2
 * 
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */


package com.radixdlt.api.system.generated.models;

import java.util.Objects;
import java.util.Arrays;
import java.util.Map;
import java.util.HashMap;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonTypeName;
import com.fasterxml.jackson.annotation.JsonValue;
import com.radixdlt.api.system.generated.models.AddressBookEntry;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import java.util.ArrayList;
import java.util.List;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * SystemAddressBookResponse
 */
@JsonPropertyOrder({
  SystemAddressBookResponse.JSON_PROPERTY_ENTRIES
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class SystemAddressBookResponse {
  public static final String JSON_PROPERTY_ENTRIES = "entries";
  private List<AddressBookEntry> entries = new ArrayList<>();


  public SystemAddressBookResponse entries(List<AddressBookEntry> entries) {
    this.entries = entries;
    return this;
  }

  public SystemAddressBookResponse addEntriesItem(AddressBookEntry entriesItem) {
    this.entries.add(entriesItem);
    return this;
  }

   /**
   * Get entries
   * @return entries
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_ENTRIES)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public List<AddressBookEntry> getEntries() {
    return entries;
  }


  @JsonProperty(JSON_PROPERTY_ENTRIES)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setEntries(List<AddressBookEntry> entries) {
    this.entries = entries;
  }


  /**
   * Return true if this SystemAddressBookResponse object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    SystemAddressBookResponse systemAddressBookResponse = (SystemAddressBookResponse) o;
    return Objects.equals(this.entries, systemAddressBookResponse.entries);
  }

  @Override
  public int hashCode() {
    return Objects.hash(entries);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class SystemAddressBookResponse {\n");
    sb.append("    entries: ").append(toIndentedString(entries)).append("\n");
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

