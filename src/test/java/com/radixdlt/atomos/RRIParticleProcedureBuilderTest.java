package com.radixdlt.atomos;

import static org.assertj.core.api.AssertionsForInterfaceTypes.assertThat;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

import com.radixdlt.atomos.RRI;
import com.radixdlt.atomos.RRIParticle;
import com.radixdlt.atomos.RRIParticleProcedureBuilder;
import com.radixdlt.atomos.RadixAddress;
import com.radixdlt.atoms.Particle;
import com.radixdlt.common.EUID;
import com.radixdlt.common.Pair;
import com.radixdlt.constraintmachine.AtomMetadata;
import com.radixdlt.constraintmachine.ParticleProcedure;
import com.radixdlt.constraintmachine.ParticleProcedure.ProcedureResult;
import java.util.Stack;
import java.util.concurrent.atomic.AtomicReference;
import org.junit.Test;

public class RRIParticleProcedureBuilderTest {
	private static class CustomParticle extends Particle {
		private RRI rri;

		RRI getRRI() {
			return rri;
		}

		@Override
		public String toString() {
			return rri.toString();
		}
	}

	@Test
	public void when_an_rri_is_consumed_with_a_corresponding_particle__then_an_input_should_succeed_and_stack_is_empty() {
		ParticleProcedure procedure = new RRIParticleProcedureBuilder()
			.add(CustomParticle.class, CustomParticle::getRRI)
			.build();

		RadixAddress address = mock(RadixAddress.class);
		when(address.getUID()).thenReturn(EUID.ONE);
		RRI rri = mock(RRI.class);
		when(rri.getAddress()).thenReturn(address);

		AtomMetadata metadata = mock(AtomMetadata.class);
		when(metadata.isSignedBy(eq(address))).thenReturn(true);

		CustomParticle customParticle = mock(CustomParticle.class);
		when(customParticle.getRRI()).thenReturn(rri);

		ProcedureResult result = procedure.execute(
			new RRIParticle(rri),
			new AtomicReference<>(),
			customParticle,
			new AtomicReference<>(),
			metadata
		);

		assertThat(result).isEqualTo(ProcedureResult.POP_INPUT_OUTPUT);
	}

	@Test
	public void when_an_rri_is_consumed_without_a_corresponding_particle__then_input_should_fail() {
		ParticleProcedure procedure = new RRIParticleProcedureBuilder().build();

		RadixAddress address = mock(RadixAddress.class);
		when(address.getUID()).thenReturn(EUID.ONE);
		RRI rri = mock(RRI.class);
		when(rri.getAddress()).thenReturn(address);

		AtomMetadata metadata = mock(AtomMetadata.class);
		when(metadata.isSignedBy(eq(address))).thenReturn(true);

		Stack<Pair<Particle, Object>> stack = new Stack<>();
		stack.push(Pair.of(mock(CustomParticle.class), null));
		ProcedureResult result = procedure.execute(
			new RRIParticle(rri),
			new AtomicReference<>(),
			mock(CustomParticle.class),
			new AtomicReference<>(),
			metadata
		);

		assertThat(result).isEqualTo(ProcedureResult.ERROR);
	}
}