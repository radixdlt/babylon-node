package com.radixdlt.environment.deterministic.network;

import com.radixdlt.consensus.Vote;

import java.util.Random;

public final class MessageMutators {
    private static class VoteDropper implements MessageMutator {
        private final Random random = new Random(12345);
        private final double dropRate;

        private VoteDropper(double dropRate) {
            this.dropRate = dropRate;
        }

        @Override
        public boolean mutate(ControlledMessage message, MessageQueue queue) {
            return message.message() instanceof Vote && random.nextDouble() <= dropRate;
        }
    }

    private MessageMutators() {
        throw new IllegalStateException("Cannot instatiate.");
    }

    public static MessageMutator voteDropper(double dropRate) {
        return new VoteDropper(dropRate);
    }
}
