# SBOR

See the docs on the [original implementation in Rust](https://github.com/radixdlt/radixdlt-scrypto/tree/v0.4.1/sbor).

Currently in this Java repo, we have only implemented SBOR with Schema. Schemaless SBOR is to come.

## Java SBOR with Schema

The encoding and decoding of a Type `T` is defined by a class implementing `Codec<T>`.

These implementations live on a `CodecMap`, which is responsible for storing registered "Codec Builders", caching built codecs, and resolving a codec for a type.

In Java, we have to fight "Type Erasure" - so we need to explicitly provide types into interfaces to resolve against.

Consider a nested Type, `T<X<Y<A,B>>>`. Then, for SBOr, this is interpreted as:
* Having a "rawType" (AKA "baseClass") of `T.class`
* Having an "explicitType" (AKA "type" or "type token") of `new TypeToken<T<X<Y<A,B>>>>() {}` - we use Google Guava's TypeToken to capture the explicit type information.

When registering a codec creator, these are registered against the "rawType", and may either be a standard or generic codec creator, for .

For examples of a standard codec creator, see `SimpleRecord.java`. For examples of a generic codec creator, see `ResultCodec.java`.

An instantiation of `Sbor` wraps a `CodecMap`, and some configuration choices (eg whether to include/expect Type Ids in the SBOR bytes).

### Defining a codec creator

A codec creator is typically registered against a new custom type - either a class/record or an enum.

Examples of a StructCodec (from `SimpleRecord.java`):

```
  public static void registerCodec(CodecMap codecMap) {
    codecMap.register(
        SimpleRecord.class,
        (codecs) ->
            StructCodec.with(
                SimpleRecord::new,
                codecs.of(int.class),
                codecs.of(String.class),
                codecs.of(new TypeToken<Either<Long, String>>() {}),
                codecs.of(new TypeToken<Option<Boolean>>() {}),
                (r, encoder) -> encoder.encode(r.first, r.second, r.third, r.fourth)));
  }
```

And, for Enums represented as sealed interfaces (see eg `SimpleEnum.java`):

```

  static void registerCodec(CodecMap codecMap) {
    codecMap.registerForSealedClassAndSubclasses(
        SimpleEnum.class,
        (codecs) ->
            EnumCodec.fromEntries(
                EnumEntry.with(
                    A.class,
                    A::new,
                    codecs.of(int.class),
                    codecs.of(String.class),
                    (t, encoder) -> encoder.encode(t.first, t.second)),
                EnumEntry.with(
                    B.class,
                    B::new,
                    codecs.of(new TypeToken<Either<Long, String>>() {}),
                    (t, encoder) -> encoder.encode(t.param1))));
  }
```

### Creating an explicit codec

You may wish to create an explicit codec for some use cases, particularly for things that are only used in one place and don't need registering.

If you wish to simple create a codec, you can create it like this, from an `Sbor` implementation.

```
  static Codec<Tuple.Tuple2<Integer, List<String>>> addMethodParametersCodec = new Sbor(true)
  .createCodec(codecs -> 
    TupleCodec.of(
      codecs.of(Integer.class),
      codecs.of(new TypeToken<List<String>>() {})
    )
  );
  ```

  You could also save the `static TypeToken<Tuple.Tuple2<Integer, List<String>>> addMethodParametersType = new TypeToken<>() {}` and pass that into `Sbor.decode(bytes, addMethodParametersType)`.

### Using an application-wide Sbor / CodecMap

You should typically define a static CodecMap on some class, and a static function to create that CodecMap.

In that method, you should include all the codec registrations.

Something like this:

```
public final class StateManagerSbor {

  public static final Sbor sbor = createSborForStateManager();

  private static Sbor createSborForStateManager() {
    return new Sbor(true, new CodecMap().register(StateManagerSbor::registerCodecsWithCodecMap));
  }

  public static void registerCodecsWithCodecMap(CodecMap codecMap) {
    RustMempoolConfig.registerCodec(codecMap);
    StateManagerConfig.registerCodec(codecMap);
    Transaction.registerCodec(codecMap);
    TID.registerCodec(codecMap);
    StateManagerRuntimeError.registerCodec(codecMap);
    MempoolError.registerCodec(codecMap);
  }
}
```
