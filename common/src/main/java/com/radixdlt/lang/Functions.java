package com.radixdlt.lang;

/**
 * Collection of basic functions for various use cases.
 */
public interface Functions {
    /**
     * Function with no parameters (supplier). Provided for consistency.
     */
    @FunctionalInterface
    interface FN0<R> {
        R apply();
    }

    /**
     * Function with one parameter.
     */
    @FunctionalInterface
    interface FN1<R, T1> {
        R apply(T1 param1);

        default <N> FN1<N, T1> then(FN1<N, R> function) {
            return v1 -> function.apply(apply(v1));
        }

        default <N> FN1<R, N> before(FN1<T1, N> function) {
            return v1 -> apply(function.apply(v1));
        }

        static <T> FN1<T, T> id() {
            return Functions::id;
        }
    }

    /**
     * Function with two parameters.
     */
    @FunctionalInterface
    interface FN2<R, T1, T2> {
        R apply(T1 param1, T2 param2);
    }

    /**
     * Function with three parameters.
     */
    @FunctionalInterface
    interface FN3<R, T1, T2, T3> {
        R apply(T1 param1, T2 param2, T3 param3);
    }

    /**
     * Function with four parameters.
     */
    @FunctionalInterface
    interface FN4<R, T1, T2, T3, T4> {
        R apply(T1 param1, T2 param2, T3 param3, T4 param4);
    }

    /**
     * Function with five parameters.
     */
    @FunctionalInterface
    interface FN5<R, T1, T2, T3, T4, T5> {
        R apply(T1 param1, T2 param2, T3 param3, T4 param4, T5 param5);
    }

    /**
     * Function with six parameters.
     */
    @FunctionalInterface
    interface FN6<R, T1, T2, T3, T4, T5, T6> {
        R apply(T1 param1, T2 param2, T3 param3, T4 param4, T5 param5, T6 param6);
    }

    /**
     * Function with seven parameters.
     */
    @FunctionalInterface
    interface FN7<R, T1, T2, T3, T4, T5, T6, T7> {
        R apply(T1 param1, T2 param2, T3 param3, T4 param4, T5 param5, T6 param6, T7 param7);
    }

    /**
     * Function with eight parameters.
     */
    @FunctionalInterface
    interface FN8<R, T1, T2, T3, T4, T5, T6, T7, T8> {
        R apply(T1 param1, T2 param2, T3 param3, T4 param4, T5 param5, T6 param6, T7 param7, T8 param8);
    }

    /**
     * Function with nine parameters.
     */
    @FunctionalInterface
    interface FN9<R, T1, T2, T3, T4, T5, T6, T7, T8, T9> {
        R apply(T1 param1, T2 param2, T3 param3, T4 param4, T5 param5, T6 param6, T7 param7, T8 param8, T9 param9);
    }

    @FunctionalInterface
    interface FN10<R, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10> {
        R apply(T1 param1, T2 param2, T3 param3, T4 param4, T5 param5, T6 param6, T7 param7, T8 param8, T9 param9, T10 param10);
    }

    @FunctionalInterface
    interface FN11<R, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11> {
        R apply(T1 param1, T2 param2, T3 param3, T4 param4, T5 param5, T6 param6, T7 param7, T8 param8, T9 param9, T10 param10, T11 param11);
    }

    @FunctionalInterface
    interface FN12<R, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12> {
        R apply(T1 param1, T2 param2, T3 param3, T4 param4, T5 param5, T6 param6, T7 param7, T8 param8, T9 param9, T10 param10, T11 param11, T12 param12);
    }

    /**
     * Universal identity function.
     */
    static <T> T id(T value) {
        return value;
    }

    /**
     * Supplier which can throw an exception.
     */
    @FunctionalInterface
    interface ThrowingSupplier<T> {
        T get() throws Throwable;
    }

    /**
     * Consumer with three parameters.
     */
    @FunctionalInterface
    interface TriConsumer<T, K, V> {
        void accept(T t, K k, V v);
    }

    /**
     * Function with variable argument list.
     */
    @FunctionalInterface
    interface FNx<R> {
        R apply(Object... values);
    }

    /**
     * Universal consumers of values which do nothing with input values. Useful for cases when API requires function, but there is no need to do
     * anything with the received values.
     */
    static <T1> void unitFn() {
    }

    static <T1> void unitFn(T1 value) {
    }

    static <T1, T2> void unitFn(T1 param1, T2 param2) {
    }

    static <T1, T2, T3> void unitFn(T1 param1, T2 param2, T3 param3) {
    }

    static <T1, T2, T3, T4> void unitFn(T1 param1, T2 param2, T3 param3, T4 param4) {
    }

    static <T1, T2, T3, T4, T5> void unitFn(T1 param1, T2 param2, T3 param3, T4 param4, T5 param5) {
    }

    static <T1, T2, T3, T4, T5, T6> void unitFn(T1 param1, T2 param2, T3 param3, T4 param4, T5 param5, T6 param6) {
    }

    static <T1, T2, T3, T4, T5, T6, T7> void unitFn(T1 param1, T2 param2, T3 param3, T4 param4, T5 param5, T6 param6, T7 param7) {
    }

    static <T1, T2, T3, T4, T5, T6, T7, T8> void unitFn(T1 param1, T2 param2, T3 param3, T4 param4, T5 param5, T6 param6, T7 param7, T8 param8) {
    }

    static <T1, T2, T3, T4, T5, T6, T7, T8, T9> void unitFn(T1 param1, T2 param2, T3 param3, T4 param4, T5 param5, T6 param6, T7 param7, T8 param8, T9 param9) {
    }

    static <R, T1> R toNull(T1 value) {
        return null;
    }

    static <T1> boolean toTrue(T1 value) {
        return true;
    }

    static <T1> boolean toFalse(T1 value) {
        return false;
    }
}
