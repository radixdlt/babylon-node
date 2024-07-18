use std::{iter, thread};
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;
use std::time::{Duration, Instant};

use postgres::{Client, NoTls};
use radix_common::address::AddressBech32Encoder;
use radix_substate_store_queries::typed_substate_layout::{to_typed_substate_key, to_typed_substate_value, TypedSubstateKey, TypedSubstateValue};

use state_manager::{StateVersion, SubstateChangeAction, SubstateReference};
use state_manager::traits::*;

use crate::core_api::CoreApiState;
use crate::da::db::*;
use crate::da::processing_context::ProcessingContext;
use crate::da::processors::*;
use crate::da::scan_result::*;

mod scan_result;
mod db;
mod lru_like_dictionary;
mod processors;
mod processing_context;

// TODO #1: reduce .clone()ing
// TODO #2: reduce primitive variables usage
// TODO #3: use separate channels for scanning and DB persistence; possibly use even separate channel for read rocksdb + create missing definitions?
// TODO #4: eliminate all of those Box<dyn Error> and "bleee" errors
// TODO #5: actually does it even makes sense to process TX stream in batches if we stream it from the database? on a one hand batching eliminates tone of network round-trips of "find existing/most-recent" queries but maybe it's not worth it after all? batch saves are definitely valuable!
// TODO #6: implement From trait to simplify lookup type creation

pub fn da_main(core_api_state: CoreApiState) -> Result<(), Box<dyn Error>> {
    let mut postgres_db = Client::connect("host=localhost user=db_dev_superuser password=db_dev_password dbname=rust_agg", NoTls)?;
    let sequences = DbSequences::new(&mut postgres_db)?;
    let ledger_tip = read_ledger_tip(&mut postgres_db);
    let address_encoder = AddressBech32Encoder::new(&core_api_state.network);
    let mut processing_context = ProcessingContext::new(&mut postgres_db, ledger_tip?.unwrap_or(0));

    // TODO not sure if we can use direct access or should we use the .snapshot()
    let node_db = core_api_state.state_manager.database.access_direct();
    let limit = 1000;

    loop {
        let running = core_api_state.da_state.lock().unwrap().should_run;

        if !running {
            break;
        }

        processing_context.stopwatch.replace(Instant::now());

        // TODO similarly to the Core API we'll fetch batches of 1000 TXs but this is definitely FAR from being most performant approach
        let bundles_iter = node_db.get_committed_transaction_bundle_iter(StateVersion::of(processing_context.ledger_tip as u64 + 1));
        let proofs_iter = Box::new(iter::empty());
        let transactions_and_proofs_iter = TransactionAndProofIterator::new(bundles_iter, proofs_iter);
        let mut transaction_batch = vec![];

        for (bundle, _) in transactions_and_proofs_iter.take(limit) {
            let CommittedTransactionBundle {
                state_version,
                raw: _,
                receipt,
                identifiers: _,
            } = bundle;

            let mut substate_changes = vec![];

            for (substate_reference, action) in receipt.on_ledger.state_changes.substate_level_changes.iter() {
                let SubstateReference(node_id, partition_number, substate_key) = &substate_reference;
                let entity_type = node_id.entity_type().ok_or("bleee 4")?;
                let node_address = address_encoder.encode(node_id.as_ref())?;
                let key = to_typed_substate_key(entity_type, *partition_number, substate_key)?;

                // let (new, previous) = match action {
                //     SubstateChangeAction::Create { new } => (Some(to_typed_substate_value(&key, new)?), None),
                //     SubstateChangeAction::Update { previous, new } => (Some(to_typed_substate_value(&key, new)?), Some(to_typed_substate_value(&key, previous)?)),
                //     SubstateChangeAction::Delete { previous } => (None, Some(to_typed_substate_value(&key, previous)?)),
                // };

                let action = match action {
                    SubstateChangeAction::Create { new } => TypedChangeAction::Upsert {
                        new: to_typed_substate_value(&key, new)?,
                        previous: None,
                    },
                    SubstateChangeAction::Update { previous, new } => TypedChangeAction::Upsert {
                        new: to_typed_substate_value(&key, new)?,
                        previous: Some(to_typed_substate_value(&key, previous)?),
                    },
                    SubstateChangeAction::Delete { previous } => TypedChangeAction::Delete {
                        previous: to_typed_substate_value(&key, previous)?,
                    }
                };

                substate_changes.push(SubstateChange {
                    node_address,
                    key,
                    action,
                });
            }

            transaction_batch.push(Tx {
                state_version,
                substate_changes,
            });
        }

        println!("[DA][MAN][{:?}] batch of {} prepared", processing_context.elapsed(), transaction_batch.len());

        if transaction_batch.len() == 0 {
            println!("[DA][MAN] nothing to do, waiting...");

            thread::sleep(Duration::from_millis(1000));

            continue;
        }

        // second loop
        proc_execute_processors(&mut processing_context, &sequences, &transaction_batch)?;
        proc_store(&mut processing_context, &sequences)?;
        proc_fin(&mut processing_context)?;
    }

    println!("[DA][MAN] DONE");

    Ok(())
}

fn proc_execute_processors(pc: &mut ProcessingContext, seq: &DbSequences, transaction_batch: &[Tx]) -> Result<(), Box<dyn Error>> {
    // STEP 1
    let existing_entities_by_address = &mut pc.existing_entities_by_address;
    let existing_entities = &mut pc.existing_entities;
    let scan_result = scan_tx_stream(&transaction_batch)?;

    let mut entity_definitions_to_load = vec![];
    for (k, _) in &scan_result.entity_definitions {
        entity_definitions_to_load.push(k.0.clone())
    }

    // actually loads from the db
    for (_, ed) in existing_entity_definitions(&mut pc.db_conn, entity_definitions_to_load.as_slice())? {
        let ed = Rc::new(ed);
        existing_entities_by_address.put(ed.address.clone(), Rc::clone(&ed));
        existing_entities.put(ed.to_lookup(), Rc::clone(&ed));
    }

    // STEP 2
    let mut ep = EntityDefinitionProcessor::new();

    ep.process_scan_result(&seq, existing_entities_by_address, existing_entities, &scan_result)?;

    pc.increases.push(Box::new(ep));

    // STEP 3 (inlined for now)
    let most_recent_entries = &mut pc.most_recent_metadata_entry_hisotry;
    let most_recent_aggregates = &mut pc.most_recent_metadata_aggregate_history;

    let mut metadata_aggregates_to_load = vec![];
    let mut metadata_entries_to_load = vec![];

    for observed in scan_result.metadata_aggregate_history {
        if let Some(ee) = existing_entities_by_address.get(&observed.0) {
            metadata_aggregates_to_load.push(ee.id);
        }
    }

    for observed in scan_result.metadata_entry_history {
        if let Some(ee) = existing_entities_by_address.get(&observed.0) {
            metadata_entries_to_load.push(DbMetadataEntryHistoryLookup {
                entity_id: ee.id,
                key: observed.1.clone(),
            });
        }
    }

    for (lookup, aggregate) in most_recent_metadata_aggregate_history(&mut pc.db_conn, metadata_aggregates_to_load.as_slice())? {
        most_recent_aggregates.put(lookup, Rc::new(RefCell::new(aggregate)));
    }

    for (lookup, aggregate) in most_recent_metadata_entry_history(&mut pc.db_conn, metadata_entries_to_load.as_slice())? {
        most_recent_entries.put(lookup, Rc::new(aggregate));
    }

    let mut ltp = LedgerTransactionProcessor::new();
    let mut mp = MetadataProcessor::new();
    let mut new_tip = pc.ledger_tip;

    for tx in transaction_batch {
        new_tip = tx.state_version.number() as i64;

        ltp.process_tx(tx, seq)?;

        for change in &tx.substate_changes {
            mp.process_change(change, tx, seq, existing_entities_by_address, most_recent_entries, most_recent_aggregates)?;
        }
    }

    pc.increases.push(Box::new(ltp));
    pc.increases.push(Box::new(mp));

    let diff = new_tip - pc.ledger_tip;
    pc.ledger_tip = new_tip;

    println!("[DA][EXE][{:?}] processed {} TXs", pc.elapsed(), diff);

    Ok(())
}

fn proc_store(pc: &mut ProcessingContext, seq: &DbSequences) -> Result<(), Box<dyn Error>> {
    let mut db_tx = pc.db_conn.transaction()?;
    let mut cnt = 0;

    for increase in &pc.increases {
        cnt += increase.save_changes(&mut db_tx)?;
    }

    seq.persist(&mut db_tx)?;
    db_tx.commit()?;

    pc.increases.clear();

    println!("[DA][STR][{:?}] stored {} new entities", pc.elapsed(), cnt);

    Ok(())
}

fn proc_fin(pc: &mut ProcessingContext) -> Result<(), Box<dyn Error>> {
    let total = pc.existing_entities_by_address.len() + pc.most_recent_metadata_aggregate_history.len() + pc.existing_entities_by_address.len();
    let e1 = pc.existing_entities_by_address.evict(|x| x.from_state_version < pc.ledger_tip);
    let e2 = pc.most_recent_metadata_aggregate_history.evict(|x| x.borrow().from_state_version < pc.ledger_tip);
    let e3 = pc.most_recent_metadata_entry_hisotry.evict(|x| x.from_state_version < pc.ledger_tip);
    let evicted = e1 + e2 + e3;

    println!("[DA][FIN][{:?}] cleared {} out of {} LRU-like entries", pc.elapsed(), evicted, total);

    Ok(())
}

// TODO rename ProcessableXxx?
// TODO create custom enum type aggregating action+typed_key+typed_value?

pub enum TypedChangeAction {
    Upsert {
        new: TypedSubstateValue,
        previous: Option<TypedSubstateValue>
    },
    Delete {
        previous: TypedSubstateValue,
    },
}

pub struct Tx {
    pub state_version: StateVersion,
    pub substate_changes: Vec<SubstateChange>,
}

pub struct SubstateChange {
    pub node_address: String,
    pub key: TypedSubstateKey,
    pub action: TypedChangeAction,
}