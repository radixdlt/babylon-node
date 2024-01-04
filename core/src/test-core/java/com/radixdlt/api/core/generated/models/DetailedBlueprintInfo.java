/*
 * Radix Core API - Babylon
 * This API is exposed by the Babylon Radix node to give clients access to the Radix Engine, Mempool and State in the node.  The default configuration is intended for use by node-runners on a private network, and is not intended to be exposed publicly. Very heavy load may impact the node's function. The node exposes a configuration flag which allows disabling certain endpoints which may be problematic, but monitoring is advised. This configuration parameter is `api.core.flags.enable_unbounded_endpoints` / `RADIXDLT_CORE_API_FLAGS_ENABLE_UNBOUNDED_ENDPOINTS`.  This API exposes queries against the node's current state (see `/lts/state/` or `/state/`), and streams of transaction history (under `/lts/stream/` or `/stream`).  If you require queries against snapshots of historical ledger state, you may also wish to consider using the [Gateway API](https://docs-babylon.radixdlt.com/).  ## Integration and forward compatibility guarantees  Integrators (such as exchanges) are recommended to use the `/lts/` endpoints - they have been designed to be clear and simple for integrators wishing to create and monitor transactions involving fungible transfers to/from accounts.  All endpoints under `/lts/` have high guarantees of forward compatibility in future node versions. We may add new fields, but existing fields will not be changed. Assuming the integrating code uses a permissive JSON parser which ignores unknown fields, any additions will not affect existing code.  Other endpoints may be changed with new node versions carrying protocol-updates, although any breaking changes will be flagged clearly in the corresponding release notes.  All responses may have additional fields added, so clients are advised to use JSON parsers which ignore unknown fields on JSON objects. 
 *
 * The version of the OpenAPI document: v1.0.4
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
import com.radixdlt.api.core.generated.models.BlueprintCollectionInfo;
import com.radixdlt.api.core.generated.models.BlueprintEventInfo;
import com.radixdlt.api.core.generated.models.BlueprintFieldInfo;
import com.radixdlt.api.core.generated.models.BlueprintFunctionInfo;
import com.radixdlt.api.core.generated.models.BlueprintMethodInfo;
import com.radixdlt.api.core.generated.models.BlueprintNamedTypeInfo;
import com.radixdlt.api.core.generated.models.BlueprintRolesDefinition;
import com.radixdlt.api.core.generated.models.GenericTypeParameter;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import java.util.ArrayList;
import java.util.List;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;


/**
 * DetailedBlueprintInfo
 */
@JsonPropertyOrder({
  DetailedBlueprintInfo.JSON_PROPERTY_OUTER_BLUEPRINT_NAME,
  DetailedBlueprintInfo.JSON_PROPERTY_IS_TRANSIENT,
  DetailedBlueprintInfo.JSON_PROPERTY_GENERIC_TYPE_PARAMETERS,
  DetailedBlueprintInfo.JSON_PROPERTY_AVAILABLE_FEATURES,
  DetailedBlueprintInfo.JSON_PROPERTY_FIELDS,
  DetailedBlueprintInfo.JSON_PROPERTY_COLLECTIONS,
  DetailedBlueprintInfo.JSON_PROPERTY_FUNCTIONS,
  DetailedBlueprintInfo.JSON_PROPERTY_METHODS,
  DetailedBlueprintInfo.JSON_PROPERTY_ROLES,
  DetailedBlueprintInfo.JSON_PROPERTY_EVENTS,
  DetailedBlueprintInfo.JSON_PROPERTY_NAMED_TYPES
})
@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
public class DetailedBlueprintInfo {
  public static final String JSON_PROPERTY_OUTER_BLUEPRINT_NAME = "outer_blueprint_name";
  private String outerBlueprintName;

  public static final String JSON_PROPERTY_IS_TRANSIENT = "is_transient";
  private Boolean isTransient;

  public static final String JSON_PROPERTY_GENERIC_TYPE_PARAMETERS = "generic_type_parameters";
  private List<GenericTypeParameter> genericTypeParameters = new ArrayList<>();

  public static final String JSON_PROPERTY_AVAILABLE_FEATURES = "available_features";
  private List<String> availableFeatures = new ArrayList<>();

  public static final String JSON_PROPERTY_FIELDS = "fields";
  private List<BlueprintFieldInfo> fields = new ArrayList<>();

  public static final String JSON_PROPERTY_COLLECTIONS = "collections";
  private List<BlueprintCollectionInfo> collections = new ArrayList<>();

  public static final String JSON_PROPERTY_FUNCTIONS = "functions";
  private List<BlueprintFunctionInfo> functions = new ArrayList<>();

  public static final String JSON_PROPERTY_METHODS = "methods";
  private List<BlueprintMethodInfo> methods = new ArrayList<>();

  public static final String JSON_PROPERTY_ROLES = "roles";
  private BlueprintRolesDefinition roles;

  public static final String JSON_PROPERTY_EVENTS = "events";
  private List<BlueprintEventInfo> events = new ArrayList<>();

  public static final String JSON_PROPERTY_NAMED_TYPES = "named_types";
  private List<BlueprintNamedTypeInfo> namedTypes = new ArrayList<>();

  public DetailedBlueprintInfo() { 
  }

  public DetailedBlueprintInfo outerBlueprintName(String outerBlueprintName) {
    this.outerBlueprintName = outerBlueprintName;
    return this;
  }

   /**
   * A name of the outer blueprint within the same package. Only present if this one is an inner blueprint. 
   * @return outerBlueprintName
  **/
  @javax.annotation.Nullable
  @ApiModelProperty(value = "A name of the outer blueprint within the same package. Only present if this one is an inner blueprint. ")
  @JsonProperty(JSON_PROPERTY_OUTER_BLUEPRINT_NAME)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)

  public String getOuterBlueprintName() {
    return outerBlueprintName;
  }


  @JsonProperty(JSON_PROPERTY_OUTER_BLUEPRINT_NAME)
  @JsonInclude(value = JsonInclude.Include.USE_DEFAULTS)
  public void setOuterBlueprintName(String outerBlueprintName) {
    this.outerBlueprintName = outerBlueprintName;
  }


  public DetailedBlueprintInfo isTransient(Boolean isTransient) {
    this.isTransient = isTransient;
    return this;
  }

   /**
   * If true, an instantiation of this blueprint cannot be persisted (e.g. buckets and proofs are transient).
   * @return isTransient
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "If true, an instantiation of this blueprint cannot be persisted (e.g. buckets and proofs are transient).")
  @JsonProperty(JSON_PROPERTY_IS_TRANSIENT)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public Boolean getIsTransient() {
    return isTransient;
  }


  @JsonProperty(JSON_PROPERTY_IS_TRANSIENT)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setIsTransient(Boolean isTransient) {
    this.isTransient = isTransient;
  }


  public DetailedBlueprintInfo genericTypeParameters(List<GenericTypeParameter> genericTypeParameters) {
    this.genericTypeParameters = genericTypeParameters;
    return this;
  }

  public DetailedBlueprintInfo addGenericTypeParametersItem(GenericTypeParameter genericTypeParametersItem) {
    this.genericTypeParameters.add(genericTypeParametersItem);
    return this;
  }

   /**
   * Generic type parameters which need to be substituted by an object (when instantiating this blueprint). See &#x60;ObjectInstanceInfo.substituted_generic_types&#x60;. 
   * @return genericTypeParameters
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "Generic type parameters which need to be substituted by an object (when instantiating this blueprint). See `ObjectInstanceInfo.substituted_generic_types`. ")
  @JsonProperty(JSON_PROPERTY_GENERIC_TYPE_PARAMETERS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public List<GenericTypeParameter> getGenericTypeParameters() {
    return genericTypeParameters;
  }


  @JsonProperty(JSON_PROPERTY_GENERIC_TYPE_PARAMETERS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setGenericTypeParameters(List<GenericTypeParameter> genericTypeParameters) {
    this.genericTypeParameters = genericTypeParameters;
  }


  public DetailedBlueprintInfo availableFeatures(List<String> availableFeatures) {
    this.availableFeatures = availableFeatures;
    return this;
  }

  public DetailedBlueprintInfo addAvailableFeaturesItem(String availableFeaturesItem) {
    this.availableFeatures.add(availableFeaturesItem);
    return this;
  }

   /**
   * Names of the features that can be enabled for an object (when instantiating this blueprint). See &#x60;ObjectInstanceInfo.enabled_features&#x60;. 
   * @return availableFeatures
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "Names of the features that can be enabled for an object (when instantiating this blueprint). See `ObjectInstanceInfo.enabled_features`. ")
  @JsonProperty(JSON_PROPERTY_AVAILABLE_FEATURES)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public List<String> getAvailableFeatures() {
    return availableFeatures;
  }


  @JsonProperty(JSON_PROPERTY_AVAILABLE_FEATURES)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setAvailableFeatures(List<String> availableFeatures) {
    this.availableFeatures = availableFeatures;
  }


  public DetailedBlueprintInfo fields(List<BlueprintFieldInfo> fields) {
    this.fields = fields;
    return this;
  }

  public DetailedBlueprintInfo addFieldsItem(BlueprintFieldInfo fieldsItem) {
    this.fields.add(fieldsItem);
    return this;
  }

   /**
   * Fields defined by this blueprint.
   * @return fields
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "Fields defined by this blueprint.")
  @JsonProperty(JSON_PROPERTY_FIELDS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public List<BlueprintFieldInfo> getFields() {
    return fields;
  }


  @JsonProperty(JSON_PROPERTY_FIELDS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setFields(List<BlueprintFieldInfo> fields) {
    this.fields = fields;
  }


  public DetailedBlueprintInfo collections(List<BlueprintCollectionInfo> collections) {
    this.collections = collections;
    return this;
  }

  public DetailedBlueprintInfo addCollectionsItem(BlueprintCollectionInfo collectionsItem) {
    this.collections.add(collectionsItem);
    return this;
  }

   /**
   * Collections defined by this blueprint.
   * @return collections
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "Collections defined by this blueprint.")
  @JsonProperty(JSON_PROPERTY_COLLECTIONS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public List<BlueprintCollectionInfo> getCollections() {
    return collections;
  }


  @JsonProperty(JSON_PROPERTY_COLLECTIONS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setCollections(List<BlueprintCollectionInfo> collections) {
    this.collections = collections;
  }


  public DetailedBlueprintInfo functions(List<BlueprintFunctionInfo> functions) {
    this.functions = functions;
    return this;
  }

  public DetailedBlueprintInfo addFunctionsItem(BlueprintFunctionInfo functionsItem) {
    this.functions.add(functionsItem);
    return this;
  }

   /**
   * Functions defined by this blueprint.
   * @return functions
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "Functions defined by this blueprint.")
  @JsonProperty(JSON_PROPERTY_FUNCTIONS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public List<BlueprintFunctionInfo> getFunctions() {
    return functions;
  }


  @JsonProperty(JSON_PROPERTY_FUNCTIONS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setFunctions(List<BlueprintFunctionInfo> functions) {
    this.functions = functions;
  }


  public DetailedBlueprintInfo methods(List<BlueprintMethodInfo> methods) {
    this.methods = methods;
    return this;
  }

  public DetailedBlueprintInfo addMethodsItem(BlueprintMethodInfo methodsItem) {
    this.methods.add(methodsItem);
    return this;
  }

   /**
   * Methods defined by this blueprint.
   * @return methods
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "Methods defined by this blueprint.")
  @JsonProperty(JSON_PROPERTY_METHODS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public List<BlueprintMethodInfo> getMethods() {
    return methods;
  }


  @JsonProperty(JSON_PROPERTY_METHODS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setMethods(List<BlueprintMethodInfo> methods) {
    this.methods = methods;
  }


  public DetailedBlueprintInfo roles(BlueprintRolesDefinition roles) {
    this.roles = roles;
    return this;
  }

   /**
   * Get roles
   * @return roles
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "")
  @JsonProperty(JSON_PROPERTY_ROLES)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public BlueprintRolesDefinition getRoles() {
    return roles;
  }


  @JsonProperty(JSON_PROPERTY_ROLES)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setRoles(BlueprintRolesDefinition roles) {
    this.roles = roles;
  }


  public DetailedBlueprintInfo events(List<BlueprintEventInfo> events) {
    this.events = events;
    return this;
  }

  public DetailedBlueprintInfo addEventsItem(BlueprintEventInfo eventsItem) {
    this.events.add(eventsItem);
    return this;
  }

   /**
   * Events defined by this blueprint.
   * @return events
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "Events defined by this blueprint.")
  @JsonProperty(JSON_PROPERTY_EVENTS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public List<BlueprintEventInfo> getEvents() {
    return events;
  }


  @JsonProperty(JSON_PROPERTY_EVENTS)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setEvents(List<BlueprintEventInfo> events) {
    this.events = events;
  }


  public DetailedBlueprintInfo namedTypes(List<BlueprintNamedTypeInfo> namedTypes) {
    this.namedTypes = namedTypes;
    return this;
  }

  public DetailedBlueprintInfo addNamedTypesItem(BlueprintNamedTypeInfo namedTypesItem) {
    this.namedTypes.add(namedTypesItem);
    return this;
  }

   /**
   * Named types defined by this blueprint.
   * @return namedTypes
  **/
  @javax.annotation.Nonnull
  @ApiModelProperty(required = true, value = "Named types defined by this blueprint.")
  @JsonProperty(JSON_PROPERTY_NAMED_TYPES)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)

  public List<BlueprintNamedTypeInfo> getNamedTypes() {
    return namedTypes;
  }


  @JsonProperty(JSON_PROPERTY_NAMED_TYPES)
  @JsonInclude(value = JsonInclude.Include.ALWAYS)
  public void setNamedTypes(List<BlueprintNamedTypeInfo> namedTypes) {
    this.namedTypes = namedTypes;
  }


  /**
   * Return true if this DetailedBlueprintInfo object is equal to o.
   */
  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    DetailedBlueprintInfo detailedBlueprintInfo = (DetailedBlueprintInfo) o;
    return Objects.equals(this.outerBlueprintName, detailedBlueprintInfo.outerBlueprintName) &&
        Objects.equals(this.isTransient, detailedBlueprintInfo.isTransient) &&
        Objects.equals(this.genericTypeParameters, detailedBlueprintInfo.genericTypeParameters) &&
        Objects.equals(this.availableFeatures, detailedBlueprintInfo.availableFeatures) &&
        Objects.equals(this.fields, detailedBlueprintInfo.fields) &&
        Objects.equals(this.collections, detailedBlueprintInfo.collections) &&
        Objects.equals(this.functions, detailedBlueprintInfo.functions) &&
        Objects.equals(this.methods, detailedBlueprintInfo.methods) &&
        Objects.equals(this.roles, detailedBlueprintInfo.roles) &&
        Objects.equals(this.events, detailedBlueprintInfo.events) &&
        Objects.equals(this.namedTypes, detailedBlueprintInfo.namedTypes);
  }

  @Override
  public int hashCode() {
    return Objects.hash(outerBlueprintName, isTransient, genericTypeParameters, availableFeatures, fields, collections, functions, methods, roles, events, namedTypes);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append("class DetailedBlueprintInfo {\n");
    sb.append("    outerBlueprintName: ").append(toIndentedString(outerBlueprintName)).append("\n");
    sb.append("    isTransient: ").append(toIndentedString(isTransient)).append("\n");
    sb.append("    genericTypeParameters: ").append(toIndentedString(genericTypeParameters)).append("\n");
    sb.append("    availableFeatures: ").append(toIndentedString(availableFeatures)).append("\n");
    sb.append("    fields: ").append(toIndentedString(fields)).append("\n");
    sb.append("    collections: ").append(toIndentedString(collections)).append("\n");
    sb.append("    functions: ").append(toIndentedString(functions)).append("\n");
    sb.append("    methods: ").append(toIndentedString(methods)).append("\n");
    sb.append("    roles: ").append(toIndentedString(roles)).append("\n");
    sb.append("    events: ").append(toIndentedString(events)).append("\n");
    sb.append("    namedTypes: ").append(toIndentedString(namedTypes)).append("\n");
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
