package com.radixdlt.manifest;

import com.google.common.reflect.TypeToken;
import com.radixdlt.lang.Result;
import com.radixdlt.sbor.Sbor;
import com.radixdlt.sbor.codec.CodecMap;

import java.nio.charset.StandardCharsets;

public final class ManifestCompiler {

	static {
		System.loadLibrary("statemanager");
	}

	private static final Sbor sbor;

	static {
		final var codecMap = new CodecMap();
		CompileManifestError.registerCodec(codecMap);
		sbor = new Sbor(true, codecMap);
	}

	// TODO: use Network enum once it's in sync with rev2
	public static Result<byte[], CompileManifestError> compile(String manifest, String network) {
		final var manifestBytes = manifest.getBytes(StandardCharsets.UTF_8);
		final var networkBytes = network.getBytes(StandardCharsets.UTF_8);
		final var resultBytes = compile(manifestBytes, networkBytes);
		return sbor.decode(resultBytes, new TypeToken<>() {});
	}

	private static native byte[] compile(byte[] manifest, byte[] network);
}
