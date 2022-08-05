package com.radixdlt.manifest;

import org.bouncycastle.util.encoders.Hex;
import org.junit.Test;

import java.util.Locale;

import static org.junit.Assert.assertTrue;

public final class ManifestCompilerTest {

	@Test
	public void test_compile_manifest() {
		// Arrange
		// Just a bunch of random instructions, copied over from scrypto repo tests
		final var manifest = """
			CALL_METHOD ComponentAddress("account_sim1q02r73u7nv47h80e30pc3q6ylsj7mgvparm3pnsm780qgsy064") "withdraw_by_amount" Decimal("5.0") ResourceAddress("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag");
			TAKE_FROM_WORKTOP_BY_AMOUNT Decimal("2.0") ResourceAddress("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag") Bucket("xrd");
			CALL_METHOD ComponentAddress("component_sim1q2f9vmyrmeladvz0ejfttcztqv3genlsgpu9vue83mcs835hum") "buy_gumball" Bucket("xrd");
			ASSERT_WORKTOP_CONTAINS_BY_AMOUNT Decimal("3.0") ResourceAddress("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag");
			ASSERT_WORKTOP_CONTAINS ResourceAddress("resource_sim1qzhdk7tq68u8msj38r6v6yqa5myc64ejx3ud20zlh9gseqtux6");
			TAKE_FROM_WORKTOP ResourceAddress("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag") Bucket("some_xrd");
			CREATE_PROOF_FROM_BUCKET Bucket("some_xrd") Proof("proof1");
			CLONE_PROOF Proof("proof1") Proof("proof2");
			DROP_PROOF Proof("proof1");
			DROP_PROOF Proof("proof2");""";

		final var network = "LocalSimulator";

		// Act
		final var result = ManifestCompiler.compile(manifest, network);

		// Assert
		assertTrue(result.isSuccess());
		assertTrue(result.unwrap().length > 100); // Just to make sure that it's non-empty
	}

	@Test
	public void test_compile_manifest_error() {
		final var result = ManifestCompiler.compile("CLEAR_AUTH_ZONE;", "InvalidNetwork");
		assertTrue(result.isError());
		assertTrue(result.unwrapError().message().toLowerCase(Locale.ROOT).contains("network"));
	}
}
