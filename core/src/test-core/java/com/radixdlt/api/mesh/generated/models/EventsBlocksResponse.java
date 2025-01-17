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
import com.radixdlt.api.mesh.generated.models.BlockEvent;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import java.util.ArrayList;
import java.util.List;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * EventsBlocksResponse contains an ordered collection of BlockEvents and the max retrievable sequence. 
 */
@ApiModel(description = "EventsBlocksResponse contains an ordered collection of BlockEvents and the max retrievable sequence. ")
@JsonPropertyOrder({
  EventsBlocksResponse.JSON_PROPERTY_MAX_SEQUENCE,
  EventsBlocksResponse.JSON_PROPERTY_EVENTS
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class EventsBlocksResponse {
  public static final String JSON_PROPERTY_MAX_SEQUENCE = "max_sequence";
  private Long maxSequence;

  public static final String JSON_PROPERTY_EVENTS = "events";
  private List<BlockEvent> events = new ArrayList<>();

  public EventsBlocksResponse() { 
  }

  public EventsBlocksResponse maxSequence(Long maxSequence) {
    this.maxSequence = maxSequence;
    return this;
  }

   /**
   * max_sequence is the maximum available sequence number to fetch. 
   * minimum: 0
   * @return maxSequence
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(example = "5", required = true, value = "max_sequence is the maximum available sequence number to fetch. ")
  @JsonProperty(JSON_PROPERTY_MAX_SEQUENCE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Long getMaxSequence() {
    return maxSequence;
  }


  @JsonProperty(JSON_PROPERTY_MAX_SEQUENCE)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setMaxSequence(Long maxSequence) {
    this.maxSequence = maxSequence;
  }


  public EventsBlocksResponse events(List<BlockEvent> events) {
    this.events = events;
    return this;
  }

  public EventsBlocksResponse addEventsItem(BlockEvent eventsItem) {
    this.events.add(eventsItem);
    return this;
  }

   /**
   * events is an array of BlockEvents indicating the order to add and remove blocks to maintain a canonical view of blockchain state. Lightweight clients can use this event stream to update state without implementing their own block syncing logic. 
   * @return events
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "events is an array of BlockEvents indicating the order to add and remove blocks to maintain a canonical view of blockchain state. Lightweight clients can use this event stream to update state without implementing their own block syncing logic. ")
  @JsonProperty(JSON_PROPERTY_EVENTS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public List<BlockEvent> getEvents() {
    return events;
  }


  @JsonProperty(JSON_PROPERTY_EVENTS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setEvents(List<BlockEvent> events) {
    this.events = events;
  }


  /**
   * Return true if this EventsBlocksResponse object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    EventsBlocksResponse eventsBlocksResponse = (EventsBlocksResponse) o;
    return Objects.equals(this.maxSequence, eventsBlocksResponse.maxSequence) &&
        Objects.equals(this.events, eventsBlocksResponse.events);
  }

  @Override
  public int hashCode() {
    return Objects.hash(maxSequence, events);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class EventsBlocksResponse {\n");
    sb.append("    maxSequence: ").append(toIndentedString(maxSequence)).append("\n");
    sb.append("    events: ").append(toIndentedString(events)).append("\n");
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

