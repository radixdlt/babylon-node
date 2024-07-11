package com.radixdlt.safety;

import com.radixdlt.sbor.codec.CodecMap;
import com.radixdlt.sbor.codec.StructCodec;
import java.util.Arrays;

public record VertexIdDTO(byte[] idBytes) {
  public VertexIdDTO {
    if (idBytes == null) {
      throw new IllegalArgumentException("idBytes is null");
    }

    if (idBytes.length != 32) {
      throw new IllegalArgumentException("idBytes has invalid length: " + idBytes.length);
    }
  }

  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    return o instanceof VertexIdDTO that && Arrays.equals(idBytes, that.idBytes);
  }

  @Override
  public int hashCode() {
    return Arrays.hashCode(idBytes);
  }

  public static void registerCodec(CodecMap codecMap) {
    codecMap.register(
        VertexIdDTO.class,
        codecs ->
            StructCodec.with(
                VertexIdDTO::new,
                codecs.of(byte[].class),
                (t, encoder) -> encoder.encode(t.idBytes())));
  }
}
