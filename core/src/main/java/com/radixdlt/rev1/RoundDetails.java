package com.radixdlt.rev1;

import com.radixdlt.consensus.VertexWithHash;
import com.radixdlt.consensus.bft.BFTNode;

public record RoundDetails(
        long epoch,
        long roundNumber,
        long previousQcRoundNumber,
        BFTNode roundProposer,
        boolean roundWasTimeout,
        long roundTimestamp) {

    public static RoundDetails fromVertex(VertexWithHash vertex) {
        return new RoundDetails(
                vertex.getEpoch(),
                vertex.getRound().number(),
                vertex.getParentHeader().getRound().number(),
                vertex.getProposer(),
                vertex.isTimeout(),
                vertex.getWeightedTimestampOfParentQC()
        );
    }
}
