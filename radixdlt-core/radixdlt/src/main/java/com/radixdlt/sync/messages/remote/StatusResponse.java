/*
 * (C) Copyright 2021 Radix DLT Ltd
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

package com.radixdlt.sync.messages.remote;

import com.radixdlt.consensus.VerifiedLedgerHeaderAndProof;

import java.util.Objects;

/**
 * Node status response message.
 */
public final class StatusResponse {

    private final VerifiedLedgerHeaderAndProof header;

    public static StatusResponse create(VerifiedLedgerHeaderAndProof header) {
        return new StatusResponse(header);
    }

    private StatusResponse(VerifiedLedgerHeaderAndProof header) {
        this.header = header;
    }

    public VerifiedLedgerHeaderAndProof getHeader() {
        return header;
    }

    @Override
    public String toString() {
        return String.format("%s{header=%s}", this.getClass().getSimpleName(), this.header);
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) {
            return true;
        }
        if (o == null || getClass() != o.getClass()) {
            return false;
        }
        StatusResponse that = (StatusResponse) o;
        return Objects.equals(header, that.header);
    }

    @Override
    public int hashCode() {
        return Objects.hash(header);
    }
}
