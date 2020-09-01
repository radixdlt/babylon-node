/*
 * (C) Copyright 2020 Radix DLT Ltd
 *
 * Radix DLT Ltd licenses this file to you under the Apache License,
 * Version 2.0 (the "License"); you may not use this file except in
 * compliance with the License.  You may obtain a copy of the
 * License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
 * either express or implied.  See the License for the specific
 * language governing permissions and limitations under the License.
 */

package com.radixdlt.ledger;

import com.radixdlt.consensus.bft.BFTValidatorSet;
import com.radixdlt.consensus.epoch.EpochChange;
import com.radixdlt.ledger.StateComputerLedger.CommittedSender;
import java.util.Objects;

/**
 * Translates committed commands to epoch change messages
 */
public final class EpochChangeManager implements CommittedSender {
	private final EpochChangeSender epochChangeSender;

	public EpochChangeManager(EpochChangeSender epochChangeSender) {
		this.epochChangeSender = Objects.requireNonNull(epochChangeSender);
	}

	@Override
	public void sendCommitted(VerifiedCommittedCommand committedCommand, BFTValidatorSet validatorSet) {
		if (validatorSet != null) {
			EpochChange epochChange = new EpochChange(committedCommand.getProof().getHeader(), validatorSet);
			this.epochChangeSender.epochChange(epochChange);
		}
	}
}
