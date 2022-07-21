/* Copyright 2021 Radix Publishing Ltd incorporated in Jersey (Channel Islands).
 *
 * Licensed under the Radix License, Version 1.0 (the "License"); you may not use this
 * file except in compliance with the License. You may obtain a copy of the License at:
 *
 * radixfoundation.org/licenses/LICENSE-v1
 *
 * The Licensor hereby grants permission for the Canonical version of the Work to be
 * published, distributed and used under or by reference to the Licensor’s trademark
 * Radix ® and use of any unregistered trade names, logos or get-up.
 *
 * The Licensor provides the Work (and each Contributor provides its Contributions) on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied,
 * including, without limitation, any warranties or conditions of TITLE, NON-INFRINGEMENT,
 * MERCHANTABILITY, or FITNESS FOR A PARTICULAR PURPOSE.
 *
 * Whilst the Work is capable of being deployed, used and adopted (instantiated) to create
 * a distributed ledger it is your responsibility to test and validate the code, together
 * with all logic and performance of that code under all foreseeable scenarios.
 *
 * The Licensor does not make or purport to make and hereby excludes liability for all
 * and any representation, warranty or undertaking in any form whatsoever, whether express
 * or implied, to any entity or person, including any representation, warranty or
 * undertaking, as to the functionality security use, value or other characteristics of
 * any distributed ledger nor in respect the functioning or value of any tokens which may
 * be created stored or transferred using the Work. The Licensor does not warrant that the
 * Work or any use of the Work complies with any law or regulation in any territory where
 * it may be implemented or used or that it will be appropriate for any specific purpose.
 *
 * Neither the licensor nor any current or former employees, officers, directors, partners,
 * trustees, representatives, agents, advisors, contractors, or volunteers of the Licensor
 * shall be liable for any direct or indirect, special, incidental, consequential or other
 * losses of any kind, in tort, contract or otherwise (including but not limited to loss
 * of revenue, income or profits, or loss of use or data, or loss of reputation, or loss
 * of any economic or other opportunity of whatsoever nature or howsoever arising), arising
 * out of or in connection with (without limitation of any use, misuse, of any ledger system
 * or use made or its functionality or any performance or operation of any code or protocol
 * caused by bugs or programming or logic errors or otherwise);
 *
 * A. any offer, purchase, holding, use, sale, exchange or transmission of any
 * cryptographic keys, tokens or assets created, exchanged, stored or arising from any
 * interaction with the Work;
 *
 * B. any failure in a transmission or loss of any token or assets keys or other digital
 * artefacts due to errors in transmission;
 *
 * C. bugs, hacks, logic errors or faults in the Work or any communication;
 *
 * D. system software or apparatus including but not limited to losses caused by errors
 * in holding or transmitting tokens by any third-party;
 *
 * E. breaches or failure of security including hacker attacks, loss or disclosure of
 * password, loss of private key, unauthorised use or misuse of such passwords or keys;
 *
 * F. any losses including loss of anticipated savings or other benefits resulting from
 * use of the Work or any changes to the Work (however implemented).
 *
 * You are solely responsible for; testing, validating and evaluation of all operation
 * logic, functionality, security and appropriateness of using the Work for any commercial
 * or non-commercial purpose and for any reproduction or redistribution by You of the
 * Work. You assume all risks associated with Your use of the Work and the exercise of
 * permissions under this License.
 */

use crate::transaction_store::*;
use crate::types::*;
use std::collections::BTreeMap;

#[derive(Debug, PartialEq)]
pub struct InMemoryTransactionDatabase {
    next_state: TransactionStateVersion,
    in_memory_store: BTreeMap<TransactionStateVersion, Transaction>,
}

#[derive(Debug, PartialEq)]
pub enum InMemoryTransactionDatabaseGetError {
    NotFound(TransactionStateVersion),
}

impl InMemoryTransactionDatabase {
    pub fn new() -> InMemoryTransactionDatabase {
        InMemoryTransactionDatabase {
            next_state: 0,
            in_memory_store: BTreeMap::new(),
        }
    }

    pub fn alloc_state(&mut self) -> Option<TransactionStateVersion> {
        let state = self.next_state;
        let try_next = state.next();

        if let Some(next_state) = try_next {
            self.next_state = next_state;
            Some(state)
        } else {
            None
        }
    }

    pub fn store(
        &mut self,
        transaction: Transaction,
    ) -> Result<TransactionStateVersion, TransactionStoreStoreError> {
        let state = self
            .alloc_state()
            .ok_or(TransactionStoreStoreError::ExhaustedStateVersions)?;
        self.in_memory_store.insert(state, transaction);

        Ok(state)
    }

    pub fn get(&self, state: TransactionStateVersion) -> Option<Transaction> {
        self.in_memory_store.get(&state).cloned()
    }

    pub fn last_version(&self) -> Option<TransactionStateVersion> {
        self.next_state.prev()
    }
}

#[cfg(test)]
mod tests {
    use crate::transaction_store::in_memory::*;
    use crate::types::*;

    #[test]
    fn alloc_test() {
        let mut ts = InMemoryTransactionDatabase::new();

        assert_eq!(ts.next_state, 0);
        let ret = ts.alloc_state();
        assert_eq!(ret, Some(0));
        assert_eq!(ts.next_state, 1);
        let ret = ts.alloc_state();
        assert_eq!(ret, Some(1));
        assert_eq!(ts.next_state, 2);

        ts.next_state = u64::MAX;
        let ret = ts.alloc_state();
        assert_eq!(ret, None);
    }

    #[test]
    fn store_test() {
        let v1 = vec![1u8; 32];
        let v2 = vec![2u8; 32];
        let v3 = vec![3u8; 32];
        let t1 = Transaction {
            payload: v1.clone(),
            id: TId { bytes: v1 },
        };
        let t2 = Transaction {
            payload: v2.clone(),
            id: TId { bytes: v2 },
        };
        let t3 = Transaction {
            payload: v3.clone(),
            id: TId { bytes: v3 },
        };

        let mut ts = InMemoryTransactionDatabase::new();
        let rc = ts.store(t1.clone());
        assert_eq!(rc, Ok(0));
        let rc = ts.store(t2.clone());
        assert_eq!(rc, Ok(1));
        let rc = ts.store(t3.clone());
        assert_eq!(rc, Ok(2));

        ts.next_state = u64::MAX;
        let rc = ts.store(t3.clone());
        assert_eq!(rc, Err(TransactionStoreStoreError::ExhaustedStateVersions));
    }

    #[test]
    fn get_test() {
        let v1 = vec![1u8; 32];
        let v2 = vec![2u8; 32];
        let v3 = vec![3u8; 32];
        let t1 = Transaction {
            payload: v1.clone(),
            id: TId { bytes: v1 },
        };
        let t2 = Transaction {
            payload: v2.clone(),
            id: TId { bytes: v2 },
        };
        let t3 = Transaction {
            payload: v3.clone(),
            id: TId { bytes: v3 },
        };

        let mut ts = InMemoryTransactionDatabase::new();
        let rc = ts.store(t1.clone());
        assert_eq!(rc, Ok(0));
        let rc = ts.store(t2.clone());
        assert_eq!(rc, Ok(1));
        let rc = ts.store(t3.clone());
        assert_eq!(rc, Ok(2));

        let rc = ts.get(0);
        assert_eq!(rc, Some(t1.clone()));
        let rc = ts.get(1);
        assert_eq!(rc, Some(t2.clone()));
        let rc = ts.get(2);
        assert_eq!(rc, Some(t3.clone()));
        let rc = ts.get(3);
        assert_eq!(rc, None);
    }
}
