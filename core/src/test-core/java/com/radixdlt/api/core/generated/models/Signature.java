/*
 * Babylon Core API
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 0.1.0
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
import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonSubTypes;
import com.fasterxml.jackson.annotation.JsonTypeInfo;
import com.fasterxml.jackson.annotation.JsonTypeName;
import com.fasterxml.jackson.annotation.JsonValue;
import com.radixdlt.api.core.generated.models.EcdsaSecp256k1Signature;
import com.radixdlt.api.core.generated.models.EddsaEd25519Signature;
import com.radixdlt.api.core.generated.models.PublicKeyType;
import io.swagger.annotations.ApiModel;
import io.swagger.annotations.ApiModelProperty;
import com.fasterxml.jackson.annotation.JsonPropertyOrder;

import com.fasterxml.jackson.core.type.TypeReference;

import java.io.IOException;
import java.util.logging.Level;
import java.util.logging.Logger;
import java.util.ArrayList;
import java.util.Collections;
import java.util.HashSet;

import com.fasterxml.jackson.core.JsonGenerator;
import com.fasterxml.jackson.core.JsonParser;
import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.core.JsonToken;
import com.fasterxml.jackson.databind.DeserializationContext;
import com.fasterxml.jackson.databind.JsonMappingException;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.MapperFeature;
import com.fasterxml.jackson.databind.SerializerProvider;
import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import com.fasterxml.jackson.databind.annotation.JsonSerialize;
import com.fasterxml.jackson.databind.deser.std.StdDeserializer;
import com.fasterxml.jackson.databind.ser.std.StdSerializer;
import com.radixdlt.api.core.generated.client.JSON;

@javax.annotation.processing.Generated(value = "org.openapitools.codegen.languages.JavaClientCodegen")
@JsonDeserialize(using = Signature.SignatureDeserializer.class)
@JsonSerialize(using = Signature.SignatureSerializer.class)
public class Signature extends AbstractOpenApiSchema {
    private static final Logger log = Logger.getLogger(Signature.class.getName());

    public static class SignatureSerializer extends StdSerializer<Signature> {
        public SignatureSerializer(Class<Signature> t) {
            super(t);
        }

        public SignatureSerializer() {
            this(null);
        }

        @Override
        public void serialize(Signature value, JsonGenerator jgen, SerializerProvider provider) throws IOException, JsonProcessingException {
            jgen.writeObject(value.getActualInstance());
        }
    }

    public static class SignatureDeserializer extends StdDeserializer<Signature> {
        public SignatureDeserializer() {
            this(Signature.class);
        }

        public SignatureDeserializer(Class<?> vc) {
            super(vc);
        }

        @Override
        public Signature deserialize(JsonParser jp, DeserializationContext ctxt) throws IOException, JsonProcessingException {
            JsonNode tree = jp.readValueAsTree();
            Object deserialized = null;
            Signature newSignature = new Signature();
            Map<String,Object> result2 = tree.traverse(jp.getCodec()).readValueAs(new TypeReference<Map<String, Object>>() {});
            String discriminatorValue = (String)result2.get("key_type");
            switch (discriminatorValue) {
                case "EcdsaSecp256k1":
                    deserialized = tree.traverse(jp.getCodec()).readValueAs(EcdsaSecp256k1Signature.class);
                    newSignature.setActualInstance(deserialized);
                    return newSignature;
                case "EcdsaSecp256k1Signature":
                    deserialized = tree.traverse(jp.getCodec()).readValueAs(EcdsaSecp256k1Signature.class);
                    newSignature.setActualInstance(deserialized);
                    return newSignature;
                case "EddsaEd25519":
                    deserialized = tree.traverse(jp.getCodec()).readValueAs(EddsaEd25519Signature.class);
                    newSignature.setActualInstance(deserialized);
                    return newSignature;
                case "EddsaEd25519Signature":
                    deserialized = tree.traverse(jp.getCodec()).readValueAs(EddsaEd25519Signature.class);
                    newSignature.setActualInstance(deserialized);
                    return newSignature;
                default:
                    log.log(Level.WARNING, String.format("Failed to lookup discriminator value `%s` for Signature. Possible values: EcdsaSecp256k1 EcdsaSecp256k1Signature EddsaEd25519 EddsaEd25519Signature", discriminatorValue));
            }

            boolean typeCoercion = ctxt.isEnabled(MapperFeature.ALLOW_COERCION_OF_SCALARS);
            int match = 0;
            JsonToken token = tree.traverse(jp.getCodec()).nextToken();
            // deserialize EcdsaSecp256k1Signature
            try {
                boolean attemptParsing = true;
                // ensure that we respect type coercion as set on the client ObjectMapper
                if (EcdsaSecp256k1Signature.class.equals(Integer.class) || EcdsaSecp256k1Signature.class.equals(Long.class) || EcdsaSecp256k1Signature.class.equals(Float.class) || EcdsaSecp256k1Signature.class.equals(Double.class) || EcdsaSecp256k1Signature.class.equals(Boolean.class) || EcdsaSecp256k1Signature.class.equals(String.class)) {
                    attemptParsing = typeCoercion;
                    if (!attemptParsing) {
                        attemptParsing |= ((EcdsaSecp256k1Signature.class.equals(Integer.class) || EcdsaSecp256k1Signature.class.equals(Long.class)) && token == JsonToken.VALUE_NUMBER_INT);
                        attemptParsing |= ((EcdsaSecp256k1Signature.class.equals(Float.class) || EcdsaSecp256k1Signature.class.equals(Double.class)) && token == JsonToken.VALUE_NUMBER_FLOAT);
                        attemptParsing |= (EcdsaSecp256k1Signature.class.equals(Boolean.class) && (token == JsonToken.VALUE_FALSE || token == JsonToken.VALUE_TRUE));
                        attemptParsing |= (EcdsaSecp256k1Signature.class.equals(String.class) && token == JsonToken.VALUE_STRING);
                    }
                }
                if (attemptParsing) {
                    deserialized = tree.traverse(jp.getCodec()).readValueAs(EcdsaSecp256k1Signature.class);
                    // TODO: there is no validation against JSON schema constraints
                    // (min, max, enum, pattern...), this does not perform a strict JSON
                    // validation, which means the 'match' count may be higher than it should be.
                    match++;
                    log.log(Level.FINER, "Input data matches schema 'EcdsaSecp256k1Signature'");
                }
            } catch (Exception e) {
                // deserialization failed, continue
                log.log(Level.FINER, "Input data does not match schema 'EcdsaSecp256k1Signature'", e);
            }

            // deserialize EddsaEd25519Signature
            try {
                boolean attemptParsing = true;
                // ensure that we respect type coercion as set on the client ObjectMapper
                if (EddsaEd25519Signature.class.equals(Integer.class) || EddsaEd25519Signature.class.equals(Long.class) || EddsaEd25519Signature.class.equals(Float.class) || EddsaEd25519Signature.class.equals(Double.class) || EddsaEd25519Signature.class.equals(Boolean.class) || EddsaEd25519Signature.class.equals(String.class)) {
                    attemptParsing = typeCoercion;
                    if (!attemptParsing) {
                        attemptParsing |= ((EddsaEd25519Signature.class.equals(Integer.class) || EddsaEd25519Signature.class.equals(Long.class)) && token == JsonToken.VALUE_NUMBER_INT);
                        attemptParsing |= ((EddsaEd25519Signature.class.equals(Float.class) || EddsaEd25519Signature.class.equals(Double.class)) && token == JsonToken.VALUE_NUMBER_FLOAT);
                        attemptParsing |= (EddsaEd25519Signature.class.equals(Boolean.class) && (token == JsonToken.VALUE_FALSE || token == JsonToken.VALUE_TRUE));
                        attemptParsing |= (EddsaEd25519Signature.class.equals(String.class) && token == JsonToken.VALUE_STRING);
                    }
                }
                if (attemptParsing) {
                    deserialized = tree.traverse(jp.getCodec()).readValueAs(EddsaEd25519Signature.class);
                    // TODO: there is no validation against JSON schema constraints
                    // (min, max, enum, pattern...), this does not perform a strict JSON
                    // validation, which means the 'match' count may be higher than it should be.
                    match++;
                    log.log(Level.FINER, "Input data matches schema 'EddsaEd25519Signature'");
                }
            } catch (Exception e) {
                // deserialization failed, continue
                log.log(Level.FINER, "Input data does not match schema 'EddsaEd25519Signature'", e);
            }

            if (match == 1) {
                Signature ret = new Signature();
                ret.setActualInstance(deserialized);
                return ret;
            }
            throw new IOException(String.format("Failed deserialization for Signature: %d classes match result, expected 1", match));
        }

        /**
         * Handle deserialization of the 'null' value.
         */
        @Override
        public Signature getNullValue(DeserializationContext ctxt) throws JsonMappingException {
            throw new JsonMappingException(ctxt.getParser(), "Signature cannot be null");
        }
    }

    // store a list of schema names defined in oneOf
    public static final Map<String, Class<?>> schemas = new HashMap<>();

    public Signature() {
        super("oneOf", Boolean.FALSE);
    }

    public Signature(EcdsaSecp256k1Signature o) {
        super("oneOf", Boolean.FALSE);
        setActualInstance(o);
    }

    public Signature(EddsaEd25519Signature o) {
        super("oneOf", Boolean.FALSE);
        setActualInstance(o);
    }

    static {
        schemas.put("EcdsaSecp256k1Signature", EcdsaSecp256k1Signature.class);
        schemas.put("EddsaEd25519Signature", EddsaEd25519Signature.class);
        JSON.registerDescendants(Signature.class, Collections.unmodifiableMap(schemas));
        // Initialize and register the discriminator mappings.
        Map<String, Class<?>> mappings = new HashMap<String, Class<?>>();
        mappings.put("EcdsaSecp256k1", EcdsaSecp256k1Signature.class);
        mappings.put("EcdsaSecp256k1Signature", EcdsaSecp256k1Signature.class);
        mappings.put("EddsaEd25519", EddsaEd25519Signature.class);
        mappings.put("EddsaEd25519Signature", EddsaEd25519Signature.class);
        mappings.put("Signature", Signature.class);
        JSON.registerDiscriminator(Signature.class, "key_type", mappings);
    }

    @Override
    public Map<String, Class<?>> getSchemas() {
        return Signature.schemas;
    }

    /**
     * Set the instance that matches the oneOf child schema, check
     * the instance parameter is valid against the oneOf child schemas:
     * EcdsaSecp256k1Signature, EddsaEd25519Signature
     *
     * It could be an instance of the 'oneOf' schemas.
     * The oneOf child schemas may themselves be a composed schema (allOf, anyOf, oneOf).
     */
    @Override
    public void setActualInstance(Object instance) {
        if (JSON.isInstanceOf(EcdsaSecp256k1Signature.class, instance, new HashSet<Class<?>>())) {
            super.setActualInstance(instance);
            return;
        }

        if (JSON.isInstanceOf(EddsaEd25519Signature.class, instance, new HashSet<Class<?>>())) {
            super.setActualInstance(instance);
            return;
        }

        throw new RuntimeException("Invalid instance type. Must be EcdsaSecp256k1Signature, EddsaEd25519Signature");
    }

    /**
     * Get the actual instance, which can be the following:
     * EcdsaSecp256k1Signature, EddsaEd25519Signature
     *
     * @return The actual instance (EcdsaSecp256k1Signature, EddsaEd25519Signature)
     */
    @Override
    public Object getActualInstance() {
        return super.getActualInstance();
    }

    /**
     * Get the actual instance of `EcdsaSecp256k1Signature`. If the actual instance is not `EcdsaSecp256k1Signature`,
     * the ClassCastException will be thrown.
     *
     * @return The actual instance of `EcdsaSecp256k1Signature`
     * @throws ClassCastException if the instance is not `EcdsaSecp256k1Signature`
     */
    public EcdsaSecp256k1Signature getEcdsaSecp256k1Signature() throws ClassCastException {
        return (EcdsaSecp256k1Signature)super.getActualInstance();
    }

    /**
     * Get the actual instance of `EddsaEd25519Signature`. If the actual instance is not `EddsaEd25519Signature`,
     * the ClassCastException will be thrown.
     *
     * @return The actual instance of `EddsaEd25519Signature`
     * @throws ClassCastException if the instance is not `EddsaEd25519Signature`
     */
    public EddsaEd25519Signature getEddsaEd25519Signature() throws ClassCastException {
        return (EddsaEd25519Signature)super.getActualInstance();
    }

}

