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
use crate::types::{TId, Transaction};
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

struct MempoolData {
    transaction: Transaction,
    inserted: Instant,
    relayed: Option<Instant>,
}

impl MempoolData {
    fn create(transaction: Transaction) -> MempoolData {
        MempoolData {
            transaction,
            inserted: Instant::now(),
            relayed: None,
        }
    }

    fn set_relayed_time(&mut self, relayed: Instant) {
        self.relayed = Some(relayed);
    }

    fn should_relay(&self, time: Instant, initial_delay: Duration, relay_delay: Duration) -> bool {
        match self.relayed {
            None => {
                // Never been relayed. Wait for initial delay.
                time >= self.inserted + initial_delay
            }
            Some(relayed) => {
                // Repeat every relay_delay
                time >= relayed + relay_delay
            }
        }
    }
}

pub struct SimpleMempool {
    max_size: u64,
    data: HashMap<TId, MempoolData>,
}

impl SimpleMempool {
    pub fn new(mempool_config: MempoolConfig) -> SimpleMempool {
        SimpleMempool {
            max_size: mempool_config.max_size as u64,
            data: HashMap::new(),
        }
    }
}

impl Mempool for SimpleMempool {
    fn add_transaction(&mut self, transaction: Transaction) -> Result<Transaction, MempoolError> {
        let len: u64 = self.data.len().try_into().unwrap();

        if len >= self.max_size {
            return Err(MempoolError::Full {
                current_size: len,
                max_size: self.max_size,
            });
        }

        let tid = transaction.id.clone();

        match self.data.entry(tid) {
            Entry::Occupied(_) => Err(MempoolError::Duplicate),

            Entry::Vacant(e) => {
                // Insert Transaction into mempool.

                let data = MempoolData::create(transaction.clone());
                e.insert(data);

                // Return same transaction for now.
                Ok(transaction)
            }
        }
    }

    fn handle_committed_transactions(
        &mut self,
        transactions: &[Transaction],
    ) -> Result<Vec<Transaction>, MempoolError> {
        let mut removed = Vec::new();
        for t in transactions {
            if self.data.remove(&t.id).is_some() {
                removed.push(t.clone())
            };
        }
        Ok(removed)
    }

    fn get_count(&self) -> u64 {
        self.data.len().try_into().unwrap()
    }

    fn get_proposal_transactions(
        &self,
        count: u64,
        prepared: &[Transaction],
    ) -> Result<Vec<Transaction>, MempoolError> {
        let prepared_ids: HashSet<TId> = prepared.iter().map(|t| t.id.clone()).collect();

        let transactions = self
            .data
            .iter()
            .filter(|&(tid, _)| !prepared_ids.contains(tid))
            .take(count as usize)
            .map(|(_, data)| data.transaction.clone())
            .collect();

        Ok(transactions)
    }

    fn get_relay_transactions(
        &mut self,
        initial_delay_millis: u64,
        repeat_delay_millis: u64,
    ) -> Result<Vec<Transaction>, MempoolError> {
        let nowish = Instant::now();
        let initial_delay = Duration::from_millis(initial_delay_millis);
        let repeat_delay = Duration::from_millis(repeat_delay_millis);

        let mut to_relay = Vec::new();
        let relay_iter = self
            .data
            .values_mut()
            .filter(|d| d.should_relay(nowish, initial_delay, repeat_delay));

        for data in relay_iter {
            data.set_relayed_time(nowish);
            to_relay.push(data.transaction.clone());
        }

        Ok(to_relay)
    }
}

#[cfg(test)]
mod tests {
    use crate::mempool::simple::*;
    use crate::types::*;

    #[test]
    fn add_and_get_test() {
        let pl1 = vec![1u8; 32];
        let pl2 = vec![2u8; 32];
        let pl3 = vec![3u8; 32];

        let tv1 = Transaction {
            payload: pl1.clone(),
            id: TId { bytes: pl1 },
        };
        let tv2 = Transaction {
            payload: pl2.clone(),
            id: TId { bytes: pl2 },
        };
        let tv3 = Transaction {
            payload: pl3.clone(),
            id: TId { bytes: pl3 },
        };

        let mut mp = SimpleMempool::new(MempoolConfig { max_size: 2 });
        assert_eq!(mp.max_size, 2);
        assert_eq!(mp.get_count(), 0);
        let rc = mp.get_proposal_transactions(3, &Vec::new());
        assert!(rc.is_ok());
        let get = rc.unwrap();
        assert!(get.is_empty());

        let rc = mp.add_transaction(tv1.clone());
        assert!(rc.is_ok());
        assert_eq!(mp.max_size, 2);
        assert_eq!(mp.get_count(), 1);
        assert!(mp.data.contains_key(&tv1.id));
        let rc = mp.get_proposal_transactions(3, &Vec::new());
        assert!(rc.is_ok());
        let get = rc.unwrap();
        assert_eq!(get.len(), 1);
        assert!(get.contains(&tv1));

        let rc = mp.get_proposal_transactions(3, &[tv1.clone(), tv2.clone(), tv3.clone()]);
        assert!(rc.is_ok());
        let get = rc.unwrap();
        assert!(get.is_empty());

        let rc = mp.get_proposal_transactions(3, &[tv2.clone(), tv3.clone()]);
        assert!(rc.is_ok());
        let get = rc.unwrap();
        assert_eq!(get.len(), 1);
        assert!(get.contains(&tv1));

        let rc = mp.add_transaction(tv1.clone());
        assert!(rc.is_err());
        assert_eq!(rc, Err(MempoolError::Duplicate));

        let rc = mp.add_transaction(tv2.clone());
        assert!(rc.is_ok());
        assert_eq!(mp.max_size, 2);
        assert_eq!(mp.get_count(), 2);
        assert!(mp.data.contains_key(&tv1.id));
        assert!(mp.data.contains_key(&tv2.id));

        let rc = mp.get_proposal_transactions(3, &Vec::new());
        assert!(rc.is_ok());
        let get = rc.unwrap();
        assert_eq!(get.len(), 2);
        assert!(get.contains(&tv1));
        assert!(get.contains(&tv2));

        let rc =
            mp.get_proposal_transactions(3, &Vec::from([tv1.clone(), tv2.clone(), tv3.clone()]));
        assert!(rc.is_ok());
        let get = rc.unwrap();
        assert!(get.is_empty());

        let rc = mp.get_proposal_transactions(3, &Vec::from([tv2.clone(), tv3.clone()]));
        assert!(rc.is_ok());
        let get = rc.unwrap();
        assert_eq!(get.len(), 1);
        assert!(get.contains(&tv1));

        let rc = mp.get_proposal_transactions(3, &Vec::from([tv1.clone(), tv3.clone()]));
        assert!(rc.is_ok());
        let get = rc.unwrap();
        assert_eq!(get.len(), 1);
        assert!(get.contains(&tv2));

        let rc = mp.handle_committed_transactions(&Vec::from([tv1.clone()]));
        assert!(rc.is_ok());
        let rem = rc.unwrap();
        assert!(rem.contains(&tv1));
        assert_eq!(rem.len(), 1);
        assert_eq!(mp.get_count(), 1);
        assert!(mp.data.contains_key(&tv2.id));
        assert!(!mp.data.contains_key(&tv1.id));

        let rc = mp.handle_committed_transactions(&Vec::from([tv2.clone()]));
        assert!(rc.is_ok());
        let rem = rc.unwrap();
        assert!(rem.contains(&tv2));
        assert_eq!(rem.len(), 1);
        assert_eq!(mp.get_count(), 0);
        assert!(!mp.data.contains_key(&tv2.id));
        assert!(!mp.data.contains_key(&tv1.id));

        // mempool is empty. Should return no transactions.
        let rc = mp.handle_committed_transactions(&Vec::from([tv3, tv2, tv1]));
        assert!(rc.is_ok());
        let rem = rc.unwrap();
        assert!(rem.is_empty());
    }

    #[test]
    fn test_relay_delays() {
        let pl1 = vec![1u8; 32];
        let pl2 = vec![2u8; 32];
        let pl3 = vec![3u8; 32];

        let tv1 = Transaction {
            payload: pl1.clone(),
            id: TId { bytes: pl1 },
        };
        let tv2 = Transaction {
            payload: pl2.clone(),
            id: TId { bytes: pl2 },
        };
        let tv3 = Transaction {
            payload: pl3.clone(),
            id: TId { bytes: pl3 },
        };

        // TODO: Add faketime or similar library not to be time
        // dependent.
        //
        // 'delay' is high enough that should always (sigh) be
        // possible to always be able to test before 'delay' time
        // passes, short enough that tests don't take too much time.
        let delay = 200; // 1/5 second

        let mut mp = SimpleMempool::new(MempoolConfig { max_size: 3 });
        let rc = mp.add_transaction(tv1.clone());
        assert!(rc.is_ok());
        let rc = mp.add_transaction(tv2.clone());
        assert!(rc.is_ok());
        let rc = mp.add_transaction(tv3.clone());
        assert!(rc.is_ok());

        // High initial delay. Check nothing gets returned.
        let rc = mp.get_relay_transactions(delay, 0);
        assert!(rc.is_ok());
        let rel = rc.unwrap();
        assert!(rel.is_empty());

        // Now sleep for the initial and check that they are all returned.
        std::thread::sleep(Duration::from_millis(delay));
        let rc = mp.get_relay_transactions(delay, 0);
        assert!(rc.is_ok());
        let rel = rc.unwrap();
        assert!(rel.contains(&tv1));
        assert!(rel.contains(&tv2));
        assert!(rel.contains(&tv3));
        assert_eq!(rel.len(), 3);

        // With no relay delay, they should be returned again immediately.
        let rc = mp.get_relay_transactions(delay, 0);
        assert!(rc.is_ok());
        let rel = rc.unwrap();
        assert!(rel.contains(&tv1));
        assert!(rel.contains(&tv2));
        assert!(rel.contains(&tv3));
        assert_eq!(rel.len(), 3);

        // With a relay delay, nothing should be returned now.
        let rc = mp.get_relay_transactions(0, delay);
        assert!(rc.is_ok());
        let rel = rc.unwrap();
        assert!(rel.is_empty());

        // Sleep for the relay delay, and check that it is returned.
        std::thread::sleep(Duration::from_millis(delay));
        let rc = mp.get_relay_transactions(0, delay);
        assert!(rc.is_ok());
        let rel = rc.unwrap();
        assert!(rel.contains(&tv1));
        assert!(rel.contains(&tv2));
        assert!(rel.contains(&tv3));
        assert_eq!(rel.len(), 3);
    }
}
