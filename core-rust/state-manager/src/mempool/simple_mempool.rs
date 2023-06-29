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

use node_common::config::MempoolConfig;
use transaction::model::*;

use crate::mempool::*;

use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

#[derive(Clone, PartialEq, Eq)]
pub struct MempoolData {
    /// The mempool transaction.
    /// The MempoolTransaction is stored in an Arc for performance, so it's cheap to clone it as part of mempool operations.
    pub transaction: Arc<MempoolTransaction>,
    /// The instant at which the transaction was added to the mempool.
    pub added_at: Instant,
    /// The source of the transaction.
    pub source: MempoolAddSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MempoolTransaction {
    pub validated: Box<ValidatedNotarizedTransactionV1>,
    pub raw: RawNotarizedTransaction,
}

impl MempoolTransaction {
    pub fn tip_percentage(&self) -> u16 {
        self.validated
            .prepared
            .signed_intent
            .intent
            .header
            .inner
            .tip_percentage
    }
}

impl HasIntentHash for MempoolTransaction {
    fn intent_hash(&self) -> IntentHash {
        self.validated.prepared.intent_hash()
    }
}

impl HasSignedIntentHash for MempoolTransaction {
    fn signed_intent_hash(&self) -> transaction::model::SignedIntentHash {
        self.validated.prepared.signed_intent_hash()
    }
}

impl HasNotarizedTransactionHash for MempoolTransaction {
    fn notarized_transaction_hash(&self) -> NotarizedTransactionHash {
        self.validated.prepared.notarized_transaction_hash()
    }
}

impl MempoolData {
    fn create(
        transaction: Arc<MempoolTransaction>,
        added_at: Instant,
        source: MempoolAddSource,
    ) -> MempoolData {
        MempoolData {
            transaction,
            added_at,
            source,
        }
    }
}

/// A wrapper around an [`Arc<MempoolData>`] which implements ordering traits for the proposal priority.
/// The order is such that a normal iteration over a BTreeSet gives the next best proposal transactions,
/// i.e. if a < b then a is prioritized over b.
#[derive(Clone, Eq)]
pub struct MempoolDataProposalPriorityOrdering(pub Arc<MempoolData>);

impl Ord for MempoolDataProposalPriorityOrdering {
    fn cmp(&self, other: &Self) -> Ordering {
        match self
            .0
            .transaction
            .tip_percentage()
            .cmp(&other.0.transaction.tip_percentage())
        {
            Ordering::Less => Ordering::Greater,
            Ordering::Greater => Ordering::Less,
            Ordering::Equal => match self.0.added_at.cmp(&other.0.added_at) {
                Ordering::Less => Ordering::Less,
                Ordering::Greater => Ordering::Greater,
                Ordering::Equal => self
                    .0
                    .transaction
                    .notarized_transaction_hash()
                    .cmp(&other.0.transaction.notarized_transaction_hash()),
            },
        }
    }
}

impl PartialOrd for MempoolDataProposalPriorityOrdering {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for MempoolDataProposalPriorityOrdering {
    fn eq(&self, other: &Self) -> bool {
        self.0.transaction.notarized_transaction_hash()
            == other.0.transaction.notarized_transaction_hash()
    }
}

pub struct SimpleMempool {
    remaining_transaction_count: usize,
    remaining_total_transactions_size: usize,
    pub proposal_priority_index: BTreeSet<MempoolDataProposalPriorityOrdering>,
    data: HashMap<NotarizedTransactionHash, Arc<MempoolData>>,
    intent_lookup: HashMap<IntentHash, HashSet<NotarizedTransactionHash>>,
}

impl SimpleMempool {
    pub fn new(config: MempoolConfig) -> SimpleMempool {
        SimpleMempool {
            remaining_transaction_count: config.max_transaction_count,
            remaining_total_transactions_size: config.max_total_transactions_size,
            proposal_priority_index: BTreeSet::new(),
            data: HashMap::new(),
            intent_lookup: HashMap::new(),
        }
    }
}

impl SimpleMempool {
    /// Tries to add a new transaction into the mempool. Assumes the transaction is not already inside (panics otherwise).
    /// Will return either a [`Vec`] of [`MempoolData`] that was evicted in order to fit the new transaction or an error
    /// if the mempool is full and the new transaction proposal priority is not better than what already exists.
    pub fn add_transaction(
        &mut self,
        transaction: Arc<MempoolTransaction>,
        source: MempoolAddSource,
    ) -> Result<Vec<Arc<MempoolData>>, MempoolAddError> {
        let payload_hash = transaction.notarized_transaction_hash();
        let intent_hash = transaction.intent_hash();
        let transaction_size = transaction.raw.0.len();

        let target_remaining_total_transactions_size = transaction_size;
        let target_remaining_transaction_count = 1;
        let transaction_data = Arc::new(MempoolData::create(transaction, Instant::now(), source));
        let new_order_data = MempoolDataProposalPriorityOrdering(transaction_data.clone());

        let mut remaining_total_transactions_size = self.remaining_total_transactions_size;
        let mut remaining_transaction_count = self.remaining_transaction_count;
        let mut to_be_removed = Vec::new();
        let mut priority_iter = self.proposal_priority_index.iter();
        // Note: worst-case scenario is the biggest transaction that will be rejected against a mempool full of smallest
        // possible transactions. This can be mitigated with a dynamic segment tree which can do fast range sum queries,
        // in order to check rejection before actually getting the evicted transactions. With a minimum transaction of
        // 1024 bytes and current max transaction size of 1MB this should not be a problem yet.
        while remaining_total_transactions_size < target_remaining_total_transactions_size
            || remaining_transaction_count < target_remaining_transaction_count
        {
            let order_data = priority_iter.next_back();
            match order_data {
                None => {
                    panic!("Impossible to add new transaction. Mempool max size lower than transaction size!");
                }
                Some(order_data) => {
                    remaining_total_transactions_size += order_data.0.transaction.raw.0.len();
                    remaining_transaction_count += 1;
                    to_be_removed.push(order_data.0.clone());
                }
            }
        }

        if !to_be_removed.is_empty() {
            let best_to_be_removed = to_be_removed.last().unwrap();
            if new_order_data
                > MempoolDataProposalPriorityOrdering((*to_be_removed.last().unwrap()).clone())
            {
                let min_tip_percentage_required =
                    best_to_be_removed.transaction.tip_percentage() + 1;
                return Err(MempoolAddError::PriorityThresholdNotMet {
                    min_tip_percentage_required,
                    tip_percentage: transaction_data.transaction.tip_percentage(),
                });
            }
        }

        for data in to_be_removed.iter() {
            self.remove_data(data.clone());
        }

        // Update remaining resources
        self.remaining_total_transactions_size -= transaction_size;
        self.remaining_transaction_count -= 1;

        // Add new MempoolData
        if self.data.insert(payload_hash, transaction_data).is_some() {
            panic!("Transaction already inside mempool");
        }

        // Add proposal priority index
        self.proposal_priority_index.insert(new_order_data);

        // Add intent lookup
        self.intent_lookup
            .entry(intent_hash)
            .and_modify(|e| {
                e.insert(payload_hash);
            })
            .or_insert_with(|| HashSet::from([payload_hash]));

        Ok(to_be_removed)
    }

    pub fn contains_transaction(&self, payload_hash: &NotarizedTransactionHash) -> bool {
        self.data.contains_key(payload_hash)
    }

    // Internal only method. Assumes data is part of the Mempool.
    fn remove_data(&mut self, data: Arc<MempoolData>) {
        let payload_hash = &data.transaction.notarized_transaction_hash();
        let intent_hash = &data.transaction.intent_hash();

        self.remaining_transaction_count += 1;
        self.remaining_total_transactions_size += data.transaction.raw.0.len();

        self.data.remove(payload_hash);

        let payload_lookup = self
            .intent_lookup
            .get_mut(intent_hash)
            .expect("Mempool intent hash lookup out of sync on remove");

        if !payload_lookup.remove(payload_hash) {
            panic!("Mempool intent hash lookup out of sync on remove");
        }
        if payload_lookup.is_empty() {
            self.intent_lookup.remove(intent_hash);
        }

        if !self
            .proposal_priority_index
            .remove(&MempoolDataProposalPriorityOrdering(data))
        {
            panic!("Mempool priority index out of sync on remove");
        }
    }

    pub fn remove_by_payload_hash(
        &mut self,
        payload_hash: &NotarizedTransactionHash,
    ) -> Option<Arc<MempoolData>> {
        let to_remove = self.data.get(payload_hash).cloned();
        match &to_remove {
            None => {}
            Some(data) => self.remove_data(data.clone()),
        }
        to_remove
    }

    pub fn remove_by_intent_hash(&mut self, intent_hash: &IntentHash) -> Vec<Arc<MempoolData>> {
        let data: Vec<_> = self
            .intent_lookup
            .get(intent_hash)
            .iter()
            .flat_map(|payload_hashes| payload_hashes.iter())
            .map(|payload_hash| {
                println!("{:?}", payload_hash);
                self.data
                    .get(payload_hash)
                    .expect("Mempool intent hash lookup out of sync on remove by intent hash.")
                    .clone()
            })
            .collect();
        data.into_iter()
            .map(|data| {
                self.remove_data(data.clone());
                data
            })
            .collect()
    }

    pub fn get_count(&self) -> usize {
        self.data.len()
    }

    pub fn get_payload_hashes_for_intent(
        &self,
        intent_hash: &IntentHash,
    ) -> Vec<NotarizedTransactionHash> {
        match self.intent_lookup.get(intent_hash) {
            Some(payload_hashes) => payload_hashes.iter().cloned().collect(),
            None => vec![],
        }
    }

    pub fn all_hashes_iter(
        &self,
    ) -> impl Iterator<Item = (&IntentHash, &NotarizedTransactionHash)> {
        self.intent_lookup
            .iter()
            .flat_map(|(intent_hash, payload_hashes)| {
                payload_hashes
                    .iter()
                    .map(move |payload_hash| (intent_hash, payload_hash))
            })
    }

    pub fn get_payload(
        &self,
        payload_hash: &NotarizedTransactionHash,
    ) -> Option<&MempoolTransaction> {
        Some(&self.data.get(payload_hash)?.transaction)
    }

    // This method needs to be removed. Consumers of SimpleMempool should not require full state retrieval.
    pub fn get_all_transactions(&self) -> Vec<Arc<MempoolTransaction>> {
        self.data
            .values()
            .map(|transaction_data| {
                // Clone the Arc - this is relatively cheap
                transaction_data.transaction.clone()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use radix_engine::types::PublicKey;
    use radix_engine_common::types::Epoch;
    use radix_engine_interface::crypto::Secp256k1PublicKey;
    use transaction::model::*;
    use transaction::signing::secp256k1::Secp256k1Signature;

    use crate::mempool::simple_mempool::*;

    fn create_fake_pub_key() -> PublicKey {
        PublicKey::Secp256k1(Secp256k1PublicKey([0; Secp256k1PublicKey::LENGTH]))
    }

    fn create_fake_signature() -> NotarySignatureV1 {
        NotarySignatureV1(SignatureV1::Secp256k1(Secp256k1Signature(
            [0; Secp256k1Signature::LENGTH],
        )))
    }

    fn create_fake_signature_with_public_key() -> IntentSignatureV1 {
        IntentSignatureV1(SignatureWithPublicKeyV1::Secp256k1 {
            signature: Secp256k1Signature([0; Secp256k1Signature::LENGTH]),
        })
    }

    fn create_fake_notarized_transaction(
        nonce: u32,
        sigs_count: usize,
        tip_percentage: u16,
    ) -> PreparedNotarizedTransactionV1 {
        NotarizedTransactionV1 {
            signed_intent: SignedIntentV1 {
                intent: IntentV1 {
                    header: TransactionHeaderV1 {
                        network_id: 1,
                        start_epoch_inclusive: Epoch::of(1),
                        end_epoch_exclusive: Epoch::of(2),
                        nonce,
                        notary_public_key: create_fake_pub_key(),
                        notary_is_signatory: false,
                        tip_percentage,
                    },
                    instructions: InstructionsV1(vec![]),
                    blobs: BlobsV1 { blobs: vec![] },
                    attachments: AttachmentsV1 {},
                },
                intent_signatures: IntentSignaturesV1 {
                    signatures: vec![0; sigs_count]
                        .into_iter()
                        .map(|_| create_fake_signature_with_public_key())
                        .collect(),
                },
            },
            notary_signature: create_fake_signature(),
        }
        .prepare()
        .expect("Expected that it could be prepared")
    }

    fn create_fake_pending_transaction(
        nonce: u32,
        sigs_count: usize,
        tip_percentage: u16,
    ) -> Arc<MempoolTransaction> {
        Arc::new(MempoolTransaction {
            validated: Box::new(ValidatedNotarizedTransactionV1 {
                prepared: create_fake_notarized_transaction(nonce, sigs_count, tip_percentage),
                // Fake these
                encoded_instructions: vec![],
                signer_keys: vec![],
            }),
            raw: RawNotarizedTransaction(vec![]),
        })
    }

    #[test]
    fn add_and_get_test() {
        let mt1 = create_fake_pending_transaction(1, 0, 0);
        let mt2 = create_fake_pending_transaction(2, 0, 0);
        let mt3 = create_fake_pending_transaction(3, 0, 0);

        let mut mp = SimpleMempool::new(MempoolConfig {
            max_transaction_count: 5,
            max_total_transactions_size: 2 * 1024 * 1024,
        });
        assert_eq!(mp.remaining_transaction_count, 5);
        assert_eq!(mp.get_count(), 0);

        mp.add_transaction(mt1.clone(), MempoolAddSource::CoreApi)
            .unwrap();
        assert_eq!(mp.remaining_transaction_count, 4);
        assert_eq!(mp.get_count(), 1);
        assert!(mp.contains_transaction(&mt1.notarized_transaction_hash()));

        mp.add_transaction(mt2.clone(), MempoolAddSource::MempoolSync)
            .unwrap();
        assert_eq!(mp.remaining_transaction_count, 3);
        assert_eq!(mp.get_count(), 2);
        assert!(mp.contains_transaction(&mt1.notarized_transaction_hash()));
        assert!(mp.contains_transaction(&mt2.notarized_transaction_hash()));

        let rem = mp.remove_by_intent_hash(&mt1.intent_hash());
        assert!(rem.iter().any(|d| d.transaction == mt1));
        assert_eq!(rem.len(), 1);
        assert_eq!(mp.get_count(), 1);
        assert!(mp.data.contains_key(&mt2.notarized_transaction_hash()));
        assert!(!mp.data.contains_key(&mt1.notarized_transaction_hash()));

        let rem = mp.remove_by_intent_hash(&mt2.intent_hash());
        assert!(rem.iter().any(|d| d.transaction == mt2));
        assert_eq!(rem.len(), 1);
        assert_eq!(mp.get_count(), 0);
        assert!(!mp.data.contains_key(&mt2.notarized_transaction_hash()));
        assert!(!mp.data.contains_key(&mt1.notarized_transaction_hash()));

        // mempool is empty. Should return no transactions.
        assert!(mp.remove_by_intent_hash(&mt3.intent_hash()).is_empty());
        assert!(mp.remove_by_intent_hash(&mt2.intent_hash()).is_empty());
        assert!(mp.remove_by_intent_hash(&mt1.intent_hash()).is_empty());
    }

    #[test]
    fn test_intent_lookup() {
        let intent_1_payload_1 = create_fake_pending_transaction(1, 1, 0);
        let intent_1_payload_2 = create_fake_pending_transaction(1, 2, 0);
        let intent_1_payload_3 = create_fake_pending_transaction(1, 3, 0);
        let intent_2_payload_1 = create_fake_pending_transaction(2, 1, 0);
        let intent_2_payload_2 = create_fake_pending_transaction(2, 2, 0);

        let mut mp = SimpleMempool::new(MempoolConfig {
            max_transaction_count: 10,
            max_total_transactions_size: 2 * 1024 * 1024,
        });
        assert!(mp
            .add_transaction(intent_1_payload_1.clone(), MempoolAddSource::CoreApi)
            .unwrap()
            .is_empty());
        assert!(mp
            .add_transaction(intent_1_payload_2.clone(), MempoolAddSource::CoreApi)
            .unwrap()
            .is_empty());
        assert!(mp
            .add_transaction(intent_1_payload_3, MempoolAddSource::MempoolSync)
            .unwrap()
            .is_empty());
        assert!(mp
            .add_transaction(intent_2_payload_1.clone(), MempoolAddSource::CoreApi)
            .unwrap()
            .is_empty());

        assert_eq!(mp.get_count(), 4);
        assert_eq!(
            mp.get_payload_hashes_for_intent(&intent_2_payload_1.intent_hash())
                .len(),
            1
        );
        assert_eq!(
            mp.get_payload_hashes_for_intent(&intent_1_payload_1.intent_hash())
                .len(),
            3
        );
        mp.remove_by_payload_hash(&intent_1_payload_2.notarized_transaction_hash());
        assert_eq!(
            mp.get_payload_hashes_for_intent(&intent_1_payload_1.intent_hash())
                .len(),
            2
        );
        let removed_data =
            mp.remove_by_payload_hash(&intent_2_payload_2.notarized_transaction_hash());
        assert!(removed_data.is_none());
        assert_eq!(
            mp.get_payload_hashes_for_intent(&intent_2_payload_2.intent_hash())
                .len(),
            1
        );
        let removed_data =
            mp.remove_by_payload_hash(&intent_2_payload_1.notarized_transaction_hash());
        assert!(removed_data.is_some());
        assert_eq!(
            mp.get_payload_hashes_for_intent(&intent_2_payload_2.intent_hash())
                .len(),
            0
        );
        assert!(mp
            .add_transaction(intent_2_payload_1, MempoolAddSource::MempoolSync)
            .unwrap()
            .is_empty());

        mp.remove_by_intent_hash(&intent_1_payload_2.intent_hash());
        assert_eq!(
            mp.get_payload_hashes_for_intent(&intent_1_payload_1.intent_hash())
                .len(),
            0
        );

        assert!(mp
            .add_transaction(intent_2_payload_2.clone(), MempoolAddSource::CoreApi)
            .unwrap()
            .is_empty());
        assert_eq!(
            mp.get_payload_hashes_for_intent(&intent_2_payload_2.intent_hash())
                .len(),
            2
        );
        mp.remove_by_intent_hash(&intent_2_payload_2.intent_hash());
        assert_eq!(mp.get_count(), 0);
    }

    #[test]
    fn test_proposal_priority_ordering() {
        let mt1 = create_fake_pending_transaction(1, 0, 10);
        let mt2 = create_fake_pending_transaction(2, 0, 20);

        let now = Instant::now();
        let time_point = [now + Duration::from_secs(1), now + Duration::from_secs(2)];

        let md1 = Arc::new(MempoolData {
            transaction: mt1.clone(),
            added_at: time_point[0],
            source: MempoolAddSource::CoreApi,
        });

        let md2 = Arc::new(MempoolData {
            transaction: mt1,
            added_at: time_point[1],
            source: MempoolAddSource::CoreApi,
        });

        // For same tip_percentage, earliest seen transaction is prioritized.
        assert!(
            MempoolDataProposalPriorityOrdering(md1.clone())
                < MempoolDataProposalPriorityOrdering(md2.clone())
        );

        let md3 = Arc::new(MempoolData {
            transaction: mt2,
            added_at: time_point[0],
            source: MempoolAddSource::CoreApi,
        });

        // Highest tip percentage is always prioritized.
        assert!(
            MempoolDataProposalPriorityOrdering(md3.clone())
                < MempoolDataProposalPriorityOrdering(md1)
        );
        assert!(
            MempoolDataProposalPriorityOrdering(md3) < MempoolDataProposalPriorityOrdering(md2)
        );
    }

    #[test]
    fn test_proposal_priority_add_eviction() {
        let mt1 = create_fake_pending_transaction(1, 0, 10);
        let mt2 = create_fake_pending_transaction(1, 0, 20);
        let mt3 = create_fake_pending_transaction(2, 0, 20);
        let mt4 = create_fake_pending_transaction(1, 0, 30);
        let mt5 = create_fake_pending_transaction(1, 0, 40);
        let mt6 = create_fake_pending_transaction(2, 0, 40);
        let mt7 = create_fake_pending_transaction(3, 0, 40);
        let mt8 = create_fake_pending_transaction(4, 0, 40);
        let mt9 = create_fake_pending_transaction(5, 0, 40);

        let mut mp = SimpleMempool::new(MempoolConfig {
            max_transaction_count: 4,
            max_total_transactions_size: 2 * 1024 * 1024,
        });

        assert!(mp
            .add_transaction(mt4.clone(), MempoolAddSource::CoreApi)
            .unwrap()
            .is_empty());
        assert!(mp
            .add_transaction(mt2.clone(), MempoolAddSource::CoreApi)
            .unwrap()
            .is_empty());
        assert!(mp
            .add_transaction(mt3.clone(), MempoolAddSource::MempoolSync)
            .unwrap()
            .is_empty());
        assert!(mp
            .add_transaction(mt1.clone(), MempoolAddSource::CoreApi)
            .unwrap()
            .is_empty());

        let evicted = mp.add_transaction(mt5, MempoolAddSource::CoreApi).unwrap();
        assert_eq!(evicted.len(), 1);
        assert_eq!(evicted[0].transaction, mt1);

        // Note: mt2 and mt3 might need reordering whenever hashes change
        let evicted = mp.add_transaction(mt6, MempoolAddSource::CoreApi).unwrap();
        assert_eq!(evicted.len(), 1);
        assert_eq!(evicted[0].transaction, mt3);

        let evicted = mp.add_transaction(mt7, MempoolAddSource::CoreApi).unwrap();
        assert_eq!(evicted.len(), 1);
        assert_eq!(evicted[0].transaction, mt2);

        let evicted = mp.add_transaction(mt8, MempoolAddSource::CoreApi).unwrap();
        assert_eq!(evicted.len(), 1);
        assert_eq!(evicted[0].transaction, mt4);

        assert!(mp.add_transaction(mt9, MempoolAddSource::CoreApi).is_err());
    }
}
