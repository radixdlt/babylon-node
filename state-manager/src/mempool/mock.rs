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

use crate::mempool::*;
use std::collections::HashSet;

pub struct MockMempool {
    max_size: u64,
    data: HashSet<Transaction>,
}

impl MockMempool {
    pub fn new(max_size: u64) -> MockMempool {
        MockMempool {
            max_size,
            data: HashSet::new(),
        }
    }
}

impl Mempool for MockMempool {
    fn add(&mut self, transaction: Transaction) -> Result<(), MempoolError> {
        let len: u64 = self.data.len() as u64;

        if len >= self.max_size {
            return Err(MempoolError::Full {
                current_size: len,
                max_size: self.max_size,
            });
        }

        if !self.data.insert(transaction) {
            return Err(MempoolError::Duplicate);
        }

        Ok(())
    }

    fn committed(&mut self, txns: &HashSet<Transaction>) {
        for t in txns {
            self.data.remove(t);
        }
    }

    fn get_count(&self) -> u64 {
        self.data.len() as u64
    }

    fn get_txns(&self, count: u64, seen: &HashSet<Transaction>) -> HashSet<Transaction> {
        self.data
            .difference(seen)
            .take(count as usize)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::mempool::mock::*;
    use crate::types::*;

    #[test]
    fn mock_test() {
        let pl1 = vec![1u8; 32];
        let pl2 = vec![2u8; 32];
        let pl3 = vec![3u8; 32];

        let tv1 = Transaction {
            payload: pl1.clone(),
            id: TID { bytes: pl1 },
        };
        let tv2 = Transaction {
            payload: pl2.clone(),
            id: TID { bytes: pl2 },
        };
        let tv3 = Transaction {
            payload: pl3.clone(),
            id: TID { bytes: pl3 },
        };

        let mut mp = MockMempool::new(2);
        assert_eq!(mp.max_size, 2);
        assert_eq!(mp.get_count(), 0);
        let get = mp.get_txns(3, &HashSet::new());
        assert!(get.is_empty());

        let rc = mp.add(tv1.clone());
        assert!(rc.is_ok());
        assert_eq!(mp.max_size, 2);
        assert_eq!(mp.get_count(), 1);
        assert!(mp.data.contains(&tv1));
        let get = mp.get_txns(3, &HashSet::new());
        assert_eq!(get.len(), 1);
        assert!(get.contains(&tv1));

        let get = mp.get_txns(3, &HashSet::from([tv1.clone(), tv2.clone(), tv3.clone()]));
        assert!(get.is_empty());

        let get = mp.get_txns(3, &HashSet::from([tv2.clone(), tv3.clone()]));
        assert_eq!(get.len(), 1);
        assert!(get.contains(&tv1));

        let rc = mp.add(tv1.clone());
        assert!(rc.is_err());
        assert_eq!(rc, Err(MempoolError::Duplicate));

        let rc = mp.add(tv2.clone());
        assert!(rc.is_ok());
        assert_eq!(mp.max_size, 2);
        assert_eq!(mp.get_count(), 2);
        assert!(mp.data.contains(&tv1));
        assert!(mp.data.contains(&tv2));

        let get = mp.get_txns(3, &HashSet::new());
        assert_eq!(get.len(), 2);
        assert!(get.contains(&tv1));
        assert!(get.contains(&tv2));

        let get = mp.get_txns(3, &HashSet::from([tv1.clone(), tv2.clone(), tv3.clone()]));
        assert!(get.is_empty());

        let get = mp.get_txns(3, &HashSet::from([tv2.clone(), tv3.clone()]));
        assert_eq!(get.len(), 1);
        assert!(get.contains(&tv1));

        let get = mp.get_txns(3, &HashSet::from([tv1.clone(), tv3]));
        assert_eq!(get.len(), 1);
        assert!(get.contains(&tv2));

        mp.committed(&HashSet::from([tv1.clone()]));
        assert_eq!(mp.get_count(), 1);
        assert!(mp.data.contains(&tv2));
        assert!(!mp.data.contains(&tv1));

        mp.committed(&HashSet::from([tv2.clone()]));
        assert_eq!(mp.get_count(), 0);
        assert!(!mp.data.contains(&tv2));
        assert!(!mp.data.contains(&tv1));
    }
}
