package com.radixdlt.constraintmachine;

import com.google.common.reflect.TypeToken;
import com.radixdlt.atomos.Result;
import com.radixdlt.common.EUID;
import com.radixdlt.constraintmachine.TransitionProcedure.ProcedureResult;
import com.radixdlt.constraintmachine.WitnessValidator.WitnessValidatorResult;
import com.radixdlt.crypto.ECPublicKey;
import com.radixdlt.crypto.ECSignature;
import com.radixdlt.crypto.Hash;
import com.radixdlt.store.SpinStateMachine;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.function.BiFunction;
import java.util.function.Function;

/**
 * An implementation of a UTXO based constraint machine which uses Radix's atom structure.
 */
public final class ConstraintMachine {
	public static class Builder {
		private Function<Particle, Result> particleStaticCheck;
		private Function<TransitionLiteral, TransitionProcedure<Particle, UsedData, Particle, UsedData>> particleProcedures;
		private BiFunction<Particle, Particle, WitnessValidator<Particle, Particle>> witnessValidators;

		public Builder setParticleStaticCheck(Function<Particle, Result> particleStaticCheck) {
			this.particleStaticCheck = particleStaticCheck;
			return this;
		}

		public Builder setParticleProcedures(
			Function<TransitionLiteral, TransitionProcedure<Particle, UsedData, Particle, UsedData>> particleProcedures
		) {
			this.particleProcedures = particleProcedures;
			return this;
		}

		public Builder setWitnessValidators(BiFunction<Particle, Particle, WitnessValidator<Particle, Particle>> witnessValidators) {
			this.witnessValidators = witnessValidators;
			return this;
		}


		public ConstraintMachine build() {
			return new ConstraintMachine(
				particleStaticCheck,
				particleProcedures,
				witnessValidators
			);
		}
	}

	private final Function<Particle, Result> particleStaticCheck;
	private final Function<TransitionLiteral, TransitionProcedure<Particle, UsedData, Particle, UsedData>> particleProcedures;
	private final BiFunction<Particle, Particle, WitnessValidator<Particle, Particle>> witnessValidators;

	ConstraintMachine(
		Function<Particle, Result> particleStaticCheck,
		Function<TransitionLiteral, TransitionProcedure<Particle, UsedData, Particle, UsedData>> particleProcedures,
		BiFunction<Particle, Particle, WitnessValidator<Particle, Particle>> witnessValidators
	) {
		this.particleStaticCheck = particleStaticCheck;
		this.particleProcedures = particleProcedures;
		this.witnessValidators = witnessValidators;
	}

	public static final class CMValidationState {
		private Particle particleRemaining = null;
		private boolean particleRemainingIsInput;
		private UsedData particleRemainingUsed = null;
		private final Map<Particle, Spin> currentSpins;
		private final Hash witness;
		private final Map<EUID, ECSignature> signatures;
		private final Map<ECPublicKey, Boolean> isSignedByCache = new HashMap<>();

		CMValidationState(Hash witness, Map<EUID, ECSignature> signatures) {
			this.currentSpins = new HashMap<>();
			this.witness = witness;
			this.signatures = signatures;
		}

		public boolean checkSpin(Particle particle, Spin spin) {
			if (currentSpins.containsKey(particle)) {
				return false;
			}

			this.currentSpins.put(particle, spin);
			return true;
		}

		public boolean isSignedBy(ECPublicKey publicKey) {
			return this.isSignedByCache.computeIfAbsent(publicKey, this::verifySignedWith);
		}

		private boolean verifySignedWith(ECPublicKey publicKey) {
			if (signatures == null || signatures.isEmpty() || witness == null) {
				return false;
			}

			final ECSignature signature = signatures.get(publicKey.getUID());
			return signature != null && publicKey.verify(witness, signature);
		}

		boolean push(Particle p) {
			final Spin curSpin = currentSpins.get(p);
			final Spin nextSpin = SpinStateMachine.next(curSpin);
			currentSpins.put(p, nextSpin);
			return nextSpin == Spin.DOWN;
		}

		Particle getCurParticle() {
			return particleRemaining;
		}

		boolean spinClashes(boolean nextIsInput) {
			return particleRemaining != null && nextIsInput == particleRemainingIsInput;
		}

		TypeToken<? extends UsedData> getInputUsedType() {
			return particleRemaining != null && particleRemainingIsInput && particleRemainingUsed != null
				? particleRemainingUsed.getTypeToken() : TypeToken.of(VoidUsedData.class);
		}

		TypeToken<? extends UsedData> getOutputUsedType() {
			return particleRemaining != null && !particleRemainingIsInput && particleRemainingUsed != null
				? particleRemainingUsed.getTypeToken() : TypeToken.of(VoidUsedData.class);
		}

		UsedData getInputUsed() {
			return particleRemaining != null && particleRemainingIsInput ? particleRemainingUsed : null;
		}

		UsedData getOutputUsed() {
			return particleRemaining != null && !particleRemainingIsInput ? particleRemainingUsed : null;
		}

		void pop() {
			this.particleRemaining = null;
			this.particleRemainingUsed = null;
		}

		void popAndReplace(Particle particle, boolean isInput, UsedData particleRemainingUsed) {
			this.particleRemaining = particle;
			this.particleRemainingIsInput = isInput;
			this.particleRemainingUsed = particleRemainingUsed;
		}

		void updateUsed(UsedData particleRemainingUsed) {
			this.particleRemainingUsed = particleRemainingUsed;
		}

		boolean isEmpty() {
			return this.particleRemaining == null;
		}

		@Override
		public String toString() {
			if (particleRemaining != null) {
				return "Remaining (" + (this.particleRemainingIsInput ? "input" : "output") + "): " + this.particleRemaining + "\n"
					+ "Used: " + this.particleRemainingUsed;
			} else {
				return "Remaining: " + "[empty]";
			}
		}
	}

	/**
	 * Executes a transition procedure given the next spun particle and a current validation state.
	 *
	 * @param dp pointer of the next spun particle
	 * @param validationState local state of validation
	 * @return the first error found, otherwise an empty optional
	 */
	Optional<CMError> validateParticle(CMValidationState validationState, Particle nextParticle, boolean isInput, DataPointer dp) {
		final Particle curParticle = validationState.getCurParticle();

		if (validationState.spinClashes(isInput)) {
			return Optional.of(
				new CMError(
					dp,
					CMErrorCode.PARTICLE_REGISTER_SPIN_CLASH,
					validationState
				)
			);
		}

		final Particle inputParticle = isInput ? nextParticle : curParticle;
		final Particle outputParticle = isInput ? curParticle : nextParticle;
		final TransitionLiteral transitionLiteral = new TransitionLiteral(
			inputParticle != null ? inputParticle.getClass() : VoidParticle.class,
			validationState.getInputUsedType(),
			outputParticle != null ? outputParticle.getClass() : VoidParticle.class,
			validationState.getOutputUsedType()
		);

		final TransitionProcedure<Particle, UsedData, Particle, UsedData> transitionProcedure = this.particleProcedures.apply(transitionLiteral);

		if (transitionProcedure == null) {
			if (inputParticle == null || outputParticle == null) {
				validationState.popAndReplace(nextParticle, isInput, null);
				return Optional.empty();
			}

			return Optional.of(
				new CMError(
					dp,
					CMErrorCode.MISSING_TRANSITION_PROCEDURE,
					validationState
				)
			);
		}

		final ProcedureResult result = transitionProcedure.execute(
			inputParticle,
			validationState.getInputUsed(),
			outputParticle,
			validationState.getOutputUsed()
		);
		switch (result.getCmAction()) {
			case POP_INPUT:
				if (!isInput) {
					validationState.popAndReplace(nextParticle, isInput, result.getUsed());
				} else {
					validationState.updateUsed(result.getUsed());
				}
				break;
			case POP_OUTPUT:
				if (isInput) {
					validationState.popAndReplace(nextParticle, isInput, result.getUsed());
				} else {
					validationState.updateUsed(result.getUsed());
				}
				break;
			case POP_INPUT_OUTPUT:
				if (result.getUsed() != null) {
					throw new IllegalStateException("POP_INPUT_OUTPUT must output null");
				}
				validationState.pop();
				break;
			case ERROR:
				return Optional.of(
					new CMError(
						dp,
						CMErrorCode.TRANSITION_ERROR,
						validationState,
						result.getErrorMessage()
					)
				);
		}

		final WitnessValidator<Particle, Particle> witnessValidator = this.witnessValidators.apply(inputParticle, outputParticle);
		if (witnessValidator == null) {
			throw new IllegalStateException("No witness validator for: " + inputParticle + " -> " + outputParticle);
		}
		final WitnessValidatorResult witnessValidatorResult = witnessValidator.validate(
			result.getCmAction(),
			inputParticle,
			outputParticle,
			validationState::isSignedBy
		);
		if (witnessValidatorResult.isError()) {
			return Optional.of(
				new CMError(
					dp,
					CMErrorCode.WITNESS_ERROR,
					validationState,
					witnessValidatorResult.getErrorMessage()
				)
			);
		}

		return Optional.empty();
	}

	/**
	 * Executes transition procedures and witness validators in a particle group and validates
	 * that the particle group is well formed.
	 *
	 * @return the first error found, otherwise an empty optional
	 */
	Optional<CMError> validateMicroInstructions(CMValidationState validationState, List<CMMicroInstruction> microInstructions) {
		int particleGroupIndex = 0;
		int particleIndex = 0;

		for (CMMicroInstruction cmMicroInstruction : microInstructions) {
			final DataPointer dp = DataPointer.ofParticle(particleGroupIndex, particleIndex);
			switch (cmMicroInstruction.getMicroOp()) {
				case CHECK_NEUTRAL:
				case CHECK_UP:
					final Result staticCheckResult = particleStaticCheck.apply(cmMicroInstruction.getParticle());
					if (staticCheckResult.isError()) {
						return Optional.of(new CMError(dp, CMErrorCode.INVALID_PARTICLE, validationState, staticCheckResult.getErrorMessage()));
					}

					final Spin checkSpin = cmMicroInstruction.getCheckSpin();
					boolean updated = validationState.checkSpin(cmMicroInstruction.getParticle(), checkSpin);
					if (!updated) {
						return Optional.of(new CMError(dp, CMErrorCode.INTERNAL_SPIN_CONFLICT, validationState));
					}
					break;
				case PUSH:
					final Particle nextParticle = cmMicroInstruction.getParticle();
					final boolean isInput = validationState.push(nextParticle);
					Optional<CMError> error = validateParticle(validationState, nextParticle, isInput, dp);
					if (error.isPresent()) {
						return error;
					}
					particleIndex++;
					break;
				case PARTICLE_GROUP:
					if (!validationState.isEmpty()) {
						return Optional.of(
							new CMError(
								DataPointer.ofParticleGroup(particleGroupIndex),
								CMErrorCode.UNEQUAL_INPUT_OUTPUT,
								validationState
							)
						);
					}
					particleGroupIndex++;
					particleIndex = 0;
					break;
				default:
					throw new IllegalStateException("Unknown CM Operation: " + cmMicroInstruction.getMicroOp());
			}
		}

		if (particleIndex != 0) {
			return Optional.of(new CMError(
				DataPointer.ofParticle(particleGroupIndex, particleIndex),
				CMErrorCode.MISSING_PARTICLE_GROUP,
				validationState
			));
		}

		return Optional.empty();
	}

	/**
	 * Validates a CM instruction and calculates the necessary state checks and post-validation
	 * write logic.
	 *
	 * @param cmInstruction instruction to validate
	 * @return the first error found, otherwise an empty optional
	 */
	public Optional<CMError> validate(CMInstruction cmInstruction) {
		final CMValidationState validationState = new CMValidationState(
			cmInstruction.getWitness(),
			cmInstruction.getSignatures()
		);

		return this.validateMicroInstructions(validationState, cmInstruction.getMicroInstructions());
	}
}
