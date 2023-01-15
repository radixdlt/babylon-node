package com.radixdlt.environment;

import java.util.Objects;

public final class MessageTransportType<N, T> {
    private final Class<N> nodeIdType;
    private final Class<T> messageType;

    private MessageTransportType(Class<N> nodeIdType, Class<T> messageType) {
        this.nodeIdType = Objects.requireNonNull(nodeIdType);
        this.messageType = Objects.requireNonNull(messageType);
    }

    public static <N, T> MessageTransportType<N, T> create(Class<N> nodeIdType, Class<T> messageType) {
        return new MessageTransportType<>(nodeIdType, messageType);
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (o == null || getClass() != o.getClass()) return false;
        MessageTransportType<?, ?> that = (MessageTransportType<?, ?>) o;
        return Objects.equals(nodeIdType, that.nodeIdType) && Objects.equals(messageType, that.messageType);
    }

    @Override
    public int hashCode() {
        return Objects.hash(nodeIdType, messageType);
    }
}
