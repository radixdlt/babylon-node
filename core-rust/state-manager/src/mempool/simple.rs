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
use crate::types::*;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

struct MempoolData {
    transaction: PendingTransaction,
    inserted: Instant,
    relayed: Option<Instant>,
}

impl MempoolData {
    fn create(transaction: PendingTransaction) -> MempoolData {
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
    data: HashMap<PayloadHash, MempoolData>,
    intent_lookup: HashMap<IntentHash, HashSet<PayloadHash>>,
}

impl SimpleMempool {
    pub fn new(mempool_config: MempoolConfig) -> SimpleMempool {
        SimpleMempool {
            max_size: mempool_config.max_size as u64,
            data: HashMap::new(),
            intent_lookup: HashMap::new(),
        }
    }
}

impl Mempool for SimpleMempool {
    fn add_transaction(&mut self, transaction: PendingTransaction) -> Result<(), MempoolAddError> {
        let len: u64 = self.data.len().try_into().unwrap();

        if len >= self.max_size {
            return Err(MempoolAddError::Full {
                current_size: len,
                max_size: self.max_size,
            });
        }

        let payload_hash = transaction.payload_hash.clone();
        let intent_hash = transaction.intent_hash.clone();

        match self.data.entry(payload_hash.clone()) {
            Entry::Occupied(_) => Err(MempoolAddError::Duplicate),

            Entry::Vacant(e) => {
                // Insert Transaction into vacant entry in the mempool.
                e.insert(MempoolData::create(transaction));

                // Add intent lookup
                match self.intent_lookup.entry(intent_hash) {
                    Entry::Occupied(mut e) => {
                        e.get_mut().insert(payload_hash);
                    }
                    Entry::Vacant(e) => {
                        e.insert(HashSet::from([payload_hash]));
                    }
                }

                Ok(())
            }
        }
    }

    fn handle_committed_transactions(
        &mut self,
        intent_hashes: &[IntentHash],
    ) -> Vec<PendingTransaction> {
        let mut removed_transactions = Vec::new();
        for intent_hash in intent_hashes {
            if let Some(payload_hashes) = self.intent_lookup.remove(intent_hash) {
                for payload_hash in payload_hashes {
                    let removed_option = self.data.remove(&payload_hash);
                    if let Some(mempool_data) = removed_option {
                        removed_transactions.push(mempool_data.transaction);
                    } else {
                        panic!("Mempool intent hash lookup out of sync");
                    }
                }
            }
        }
        removed_transactions
    }

    fn get_count(&self) -> u64 {
        self.data.len().try_into().unwrap()
    }

    fn get_proposal_transactions(
        &self,
        count: u64,
        prepared_ids: &HashSet<PayloadHash>,
    ) -> Vec<PendingTransaction> {
        let transactions = self
            .data
            .iter()
            .filter(|&(tid, _)| !prepared_ids.contains(tid))
            .take(count as usize)
            .map(|(_, data)| data.transaction.clone())
            .collect();

        transactions
    }

    fn get_relay_transactions(
        &mut self,
        initial_delay_millis: u64,
        repeat_delay_millis: u64,
    ) -> Vec<PendingTransaction> {
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

        to_relay
    }

    fn get_payload_hashes_for_intent(&self, intent_hash: &IntentHash) -> Vec<PayloadHash> {
        let payload_hashes = self.intent_lookup.get(intent_hash);
        if payload_hashes.is_none() {
            return vec![];
        }
        payload_hashes.unwrap().iter().cloned().collect()
    }

    fn get_all_payload_hashes(&self) -> Vec<PayloadHash> {
        self.data.keys().cloned().collect()
    }

    fn get_payload(&self, payload_hash: &PayloadHash) -> Option<&PendingTransaction> {
        Some(&self.data.get(payload_hash)?.transaction)
    }
}

#[cfg(test)]
mod tests {
    use radix_engine::types::{
        EcdsaSecp256k1PublicKey, EcdsaSecp256k1Signature, PublicKey, Signature,
    };
    use scrypto::prelude::SignatureWithPublicKey;
    use transaction::model::{
        NotarizedTransaction, SignedTransactionIntent, TransactionHeader, TransactionIntent,
        TransactionManifest,
    };

    use crate::mempool::simple::*;

    fn create_fake_pub_key() -> PublicKey {
        PublicKey::EcdsaSecp256k1(EcdsaSecp256k1PublicKey(
            [0; EcdsaSecp256k1PublicKey::LENGTH],
        ))
    }

    fn create_fake_signature() -> Signature {
        Signature::EcdsaSecp256k1(EcdsaSecp256k1Signature(
            [0; EcdsaSecp256k1Signature::LENGTH],
        ))
    }

    fn create_fake_signature_with_public_key() -> SignatureWithPublicKey {
        SignatureWithPublicKey::EcdsaSecp256k1 {
            signature: EcdsaSecp256k1Signature([0; EcdsaSecp256k1Signature::LENGTH]),
        }
    }

    fn create_fake_notarized_transaction(nonce: u64, sigs_count: usize) -> NotarizedTransaction {
        NotarizedTransaction {
            signed_intent: SignedTransactionIntent {
                intent: TransactionIntent {
                    header: TransactionHeader {
                        version: 1,
                        network_id: 1,
                        start_epoch_inclusive: 1,
                        end_epoch_exclusive: 2,
                        nonce,
                        notary_public_key: create_fake_pub_key(),
                        notary_as_signatory: false,
                        cost_unit_limit: 100,
                        tip_percentage: 0,
                    },
                    manifest: TransactionManifest {
                        instructions: vec![],
                        blobs: vec![],
                    },
                },
                intent_signatures: vec![0; sigs_count]
                    .into_iter()
                    .map(|_| create_fake_signature_with_public_key())
                    .collect(),
            },
            notary_signature: create_fake_signature(),
        }
    }

    fn create_fake_pending_transaction(nonce: u64, sigs_count: usize) -> PendingTransaction {
        let notarized_transaction = create_fake_notarized_transaction(nonce, sigs_count);
        let payload_hash = notarized_transaction.payload_hash();
        let intent_hash = notarized_transaction.intent_hash();
        PendingTransaction {
            payload: notarized_transaction,
            payload_hash,
            intent_hash,
        }
    }

    #[test]
    fn add_and_get_test() {
        let tv1 = create_fake_pending_transaction(1, 0);
        let tv2 = create_fake_pending_transaction(2, 0);
        let tv3 = create_fake_pending_transaction(3, 0);

        let mut mp = SimpleMempool::new(MempoolConfig { max_size: 2 });
        assert_eq!(mp.max_size, 2);
        assert_eq!(mp.get_count(), 0);
        let rc = mp.get_proposal_transactions(3, &HashSet::new());
        let get = rc;
        assert!(get.is_empty());

        let rc = mp.add_transaction(tv1.clone());
        assert!(rc.is_ok());
        assert_eq!(mp.max_size, 2);
        assert_eq!(mp.get_count(), 1);
        assert!(mp.data.contains_key(&tv1.payload_hash));
        let rc = mp.get_proposal_transactions(3, &HashSet::new());
        let get = rc;
        assert_eq!(get.len(), 1);
        assert!(get.contains(&tv1));

        let rc = mp.get_proposal_transactions(
            3,
            &HashSet::from([
                tv1.payload_hash.clone(),
                tv2.payload_hash.clone(),
                tv3.payload_hash.clone(),
            ]),
        );
        let get = rc;
        assert!(get.is_empty());

        let rc = mp.get_proposal_transactions(
            3,
            &HashSet::from([tv2.payload_hash.clone(), tv3.payload_hash.clone()]),
        );
        let get = rc;
        assert_eq!(get.len(), 1);
        assert!(get.contains(&tv1));

        let rc = mp.add_transaction(tv1.clone());
        assert!(rc.is_err());
        assert_eq!(rc, Err(MempoolAddError::Duplicate));

        let rc = mp.add_transaction(tv2.clone());
        assert!(rc.is_ok());
        assert_eq!(mp.max_size, 2);
        assert_eq!(mp.get_count(), 2);
        assert!(mp.data.contains_key(&tv1.payload_hash));
        assert!(mp.data.contains_key(&tv2.payload_hash));

        let rc = mp.get_proposal_transactions(3, &HashSet::new());
        let get = rc;
        assert_eq!(get.len(), 2);
        assert!(get.contains(&tv1));
        assert!(get.contains(&tv2));

        let rc = mp.get_proposal_transactions(
            3,
            &HashSet::from([
                tv1.payload_hash.clone(),
                tv2.payload_hash.clone(),
                tv3.payload_hash.clone(),
            ]),
        );
        let get = rc;
        assert!(get.is_empty());

        let rc = mp.get_proposal_transactions(
            3,
            &HashSet::from([tv2.payload_hash.clone(), tv3.payload_hash.clone()]),
        );
        let get = rc;
        assert_eq!(get.len(), 1);
        assert!(get.contains(&tv1));

        let rc = mp.get_proposal_transactions(
            3,
            &HashSet::from([tv1.payload_hash.clone(), tv3.payload_hash.clone()]),
        );
        let get = rc;
        assert_eq!(get.len(), 1);
        assert!(get.contains(&tv2));

        let rem = mp.handle_committed_transactions(&Vec::from([tv1.intent_hash.clone()]));
        assert!(rem.contains(&tv1));
        assert_eq!(rem.len(), 1);
        assert_eq!(mp.get_count(), 1);
        assert!(mp.data.contains_key(&tv2.payload_hash));
        assert!(!mp.data.contains_key(&tv1.payload_hash));

        let rem = mp.handle_committed_transactions(&Vec::from([tv2.intent_hash.clone()]));
        assert!(rem.contains(&tv2));
        assert_eq!(rem.len(), 1);
        assert_eq!(mp.get_count(), 0);
        assert!(!mp.data.contains_key(&tv2.payload_hash));
        assert!(!mp.data.contains_key(&tv1.payload_hash));

        // mempool is empty. Should return no transactions.
        let rem = mp.handle_committed_transactions(&Vec::from([
            tv3.intent_hash,
            tv2.intent_hash,
            tv1.intent_hash,
        ]));
        assert!(rem.is_empty());
    }

    #[test]
    fn test_relay_delays() {
        let tv1 = create_fake_pending_transaction(1, 0);
        let tv2 = create_fake_pending_transaction(2, 0);
        let tv3 = create_fake_pending_transaction(3, 0);

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
        let rel = rc;
        assert!(rel.is_empty());

        // Now sleep for the initial and check that they are all returned.
        std::thread::sleep(Duration::from_millis(delay));
        let rc = mp.get_relay_transactions(delay, 0);
        let rel = rc;
        assert!(rel.contains(&tv1));
        assert!(rel.contains(&tv2));
        assert!(rel.contains(&tv3));
        assert_eq!(rel.len(), 3);

        // With no relay delay, they should be returned again immediately.
        let rc = mp.get_relay_transactions(delay, 0);
        let rel = rc;
        assert!(rel.contains(&tv1));
        assert!(rel.contains(&tv2));
        assert!(rel.contains(&tv3));
        assert_eq!(rel.len(), 3);

        // With a relay delay, nothing should be returned now.
        let rc = mp.get_relay_transactions(0, delay);
        let rel = rc;
        assert!(rel.is_empty());

        // Sleep for the relay delay, and check that it is returned.
        std::thread::sleep(Duration::from_millis(delay));
        let rc = mp.get_relay_transactions(0, delay);
        let rel = rc;
        assert!(rel.contains(&tv1));
        assert!(rel.contains(&tv2));
        assert!(rel.contains(&tv3));
        assert_eq!(rel.len(), 3);
    }

    #[test]
    fn test_intents() {
        let intent_1_payload_1 = create_fake_pending_transaction(1, 1);
        let intent_1_payload_2 = create_fake_pending_transaction(1, 2);
        let intent_1_payload_3 = create_fake_pending_transaction(1, 3);
        let intent_2_payload_1 = create_fake_pending_transaction(2, 1);
        let intent_2_payload_2 = create_fake_pending_transaction(2, 2);

        let mut mp = SimpleMempool::new(MempoolConfig { max_size: 10 });
        mp.add_transaction(intent_1_payload_1.clone()).unwrap();
        mp.add_transaction(intent_1_payload_2.clone()).unwrap();
        mp.add_transaction(intent_1_payload_3).unwrap();
        mp.add_transaction(intent_2_payload_1).unwrap();

        assert_eq!(mp.get_count(), 4);
        assert_eq!(
            mp.get_payload_hashes_for_intent(&intent_2_payload_2.intent_hash)
                .len(),
            1
        );
        assert_eq!(
            mp.get_payload_hashes_for_intent(&intent_1_payload_1.intent_hash)
                .len(),
            3
        );
        mp.handle_committed_transactions(&[intent_1_payload_2.intent_hash]);
        assert_eq!(
            mp.get_payload_hashes_for_intent(&intent_1_payload_1.intent_hash)
                .len(),
            0
        );

        mp.add_transaction(intent_2_payload_2.clone()).unwrap();
        assert_eq!(
            mp.get_payload_hashes_for_intent(&intent_2_payload_2.intent_hash)
                .len(),
            2
        );
        mp.handle_committed_transactions(&[intent_2_payload_2.intent_hash]);
        assert_eq!(mp.get_count(), 0);
    }
}
