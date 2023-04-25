package com.radixdlt.genesis.olympia;

import com.google.common.collect.ImmutableList;
import com.google.common.hash.HashCode;
import com.google.common.reflect.TypeToken;
import com.radixdlt.crypto.ECKeyPair;
import com.radixdlt.environment.deterministic.network.MessageMutator;
import com.radixdlt.genesis.GenesisData;
import com.radixdlt.genesis.GenesisData2;
import com.radixdlt.genesis.GenesisResource;
import com.radixdlt.genesis.GenesisValidator;
import com.radixdlt.genesis.olympia.state.OlympiaStateIR;
import com.radixdlt.genesis.olympia.state.OlympiaStateIRDeserializer;
import com.radixdlt.harness.deterministic.DeterministicTest;
import com.radixdlt.harness.deterministic.PhysicalNodeConfig;
import com.radixdlt.lang.Option;
import com.radixdlt.lang.Tuple;
import com.radixdlt.mempool.MempoolRelayConfig;
import com.radixdlt.modules.FunctionalRadixNodeModule;
import com.radixdlt.modules.StateComputerConfig;
import com.radixdlt.networks.Network;
import com.radixdlt.rev2.ComponentAddress2;
import com.radixdlt.rev2.Decimal;
import com.radixdlt.rev2.REv2StateReader;
import com.radixdlt.rev2.ScryptoConstants;
import com.radixdlt.rev2.modules.REv2StateManagerModule;
import com.radixdlt.sbor.StateManagerSbor;
import com.radixdlt.transaction.REv2TransactionAndProofStore;
import com.radixdlt.transaction.TransactionBuilder;
import com.radixdlt.utils.UInt64;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;
import org.bouncycastle.pqc.math.linearalgebra.ByteUtils;
import org.junit.Test;
import org.xerial.snappy.Snappy;

import java.io.ByteArrayInputStream;
import java.io.File;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.util.Base64;
import java.util.Random;
import java.util.stream.Collectors;

import static com.radixdlt.environment.deterministic.network.MessageSelector.firstSelector;
import static org.assertj.core.api.Assertions.assertThat;

public final class REV2GenesisFromOlympiaTest {
	private static final Logger log = LogManager.getLogger();

	private DeterministicTest createTest() throws IOException {
		final var olympiaEndState = readOlympiaStateIRFromResources();
		final var genesisData = OlympiaStateToBabylonGenesisMapper.toGenesisData(olympiaEndState).unwrap();

		log.info("Created genesis data... chunks: {}", genesisData.chunks().size());

		final var chunksEncoded = genesisData.chunks().stream().map(chunk -> StateManagerSbor.encode(chunk, StateManagerSbor.resolveCodec(new TypeToken<>() { })))
				.toList();


		final var totalSize = chunksEncoded.stream().map(s -> s.length).reduce(0, (a, b) -> a + b);
		log.info("Sbor size bytes = {}", totalSize);

		int largestTxn = 0;
		for (var chunk: chunksEncoded) {
			if (chunk.length > largestTxn) {
				largestTxn = chunk.length;
			}
		}
		log.info("Largest txn {}", largestTxn);


		final var allChunksEncoded = StateManagerSbor.encode(genesisData.chunks(), StateManagerSbor.resolveCodec(new TypeToken<>() { }));


		/*
		final var genesisTransaction = TransactionBuilder.createGenesis(
			newGenesisData,
			UInt64.fromNonNegativeLong(1),
			UInt64.fromNonNegativeLong(10),
			UInt64.fromNonNegativeLong(1));
		 */

//		System.out.println("Genesis transaction size is " + genesisTransaction.getPayload().length + " bytes"); // 29665240    29665237

		//29665237
		// 7396614 1/100 balances
		// 5307663 above + one v no stakes

//		Files.write(new File("/home/lgasior/genesis-txns-chunks-all-200-29-res.raw").toPath(), allChunksEncoded);



		final var validatorTest = StateManagerSbor.encode(new GenesisValidator(
				ECKeyPair.generateNew().getPublicKey(),
				false,
				true,
				ImmutableList.of(Tuple.Tuple2.of("Abc", "cde")),
				ComponentAddress2.virtualEcdsaAccount(ECKeyPair.generateNew().getPublicKey().getBytes())
		), StateManagerSbor.resolveCodec(new TypeToken<>() { }));

//		Files.write(new File("/home/lgasior/genesis-validator-test.raw").toPath(), validatorTest);



		var b = new byte[29];
		new Random().nextBytes(b);
		final var resourcesTest = StateManagerSbor.encode(new GenesisResource(
				b,
				Decimal.of(2),
				ImmutableList.of(Tuple.Tuple2.of("Abc", "cde")),
				Option.some(ComponentAddress2.virtualEcdsaAccount(ECKeyPair.generateNew().getPublicKey().getBytes()))
		), StateManagerSbor.resolveCodec(new TypeToken<>() { }));

//		Files.write(new File("/home/lgasior/genesis-resource-test-byt29.raw").toPath(), resourcesTest);




		throw new RuntimeException("asD");
/*
		return DeterministicTest.builder()
				.addPhysicalNodes(PhysicalNodeConfig.createBatch(1, true))
				.messageSelector(firstSelector())
				.messageMutator(MessageMutator.dropTimeouts())
				.functionalNodeModule(
						new FunctionalRadixNodeModule(
								FunctionalRadixNodeModule.NodeStorageConfig.none(),
								false,
								FunctionalRadixNodeModule.SafetyRecoveryConfig.MOCKED,
								FunctionalRadixNodeModule.ConsensusConfig.of(1000),
								FunctionalRadixNodeModule.LedgerConfig.stateComputerNoSync(
										StateComputerConfig.rev2(
												Network.INTEGRATIONTESTNET.getId(),
												genesisTransaction,
												REv2StateManagerModule.DatabaseType.IN_MEMORY,
												StateComputerConfig.REV2ProposerConfig.mempool(
														0, 0, 0, MempoolRelayConfig.of())))));
 */
	}

	@Test
	public void state_reader_on_genesis_returns_correct_amounts() throws IOException {
		// Arrange/Act
		try (var test = createTest()) {
			test.startAllNodes();

			// Assert
			var stateReader = test.getInstance(0, REv2StateReader.class);

			var transactionStore = test.getInstance(0, REv2TransactionAndProofStore.class);
			var genesis = transactionStore.getTransactionAtStateVersion(1).unwrap();
			assertThat(genesis.newComponentAddresses())
					.contains(ScryptoConstants.FAUCET_COMPONENT_ADDRESS);

		}
	}

	private static OlympiaStateIR readOlympiaStateIRFromResources() throws IOException {
		try (var is = REV2GenesisFromOlympiaTest.class.getClassLoader().getResourceAsStream("genesis/olympia-end-state-2023-04-12.base64")) {
			final var compressed = Base64.getDecoder().decode(is.readAllBytes());
			final var uncompressed = Snappy.uncompress(compressed);
			return new OlympiaStateIRDeserializer().deserialize(new ByteArrayInputStream(uncompressed));
		}
	}
}
