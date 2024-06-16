use std::cell::RefCell;
use std::error::Error;
use std::{iter, thread};
use std::rc::Rc;
use std::time::{Duration, Instant};

use postgres::{Client, NoTls};
use radix_substate_store_queries::typed_substate_layout::{TypedMetadataModuleSubstateKey, TypedSubstateKey};

use state_manager::{StateVersion, SubstateChangeAction, SubstateReference};
use state_manager::traits::*;

use crate::core_api::{CoreApiState, create_typed_substate_key, create_typed_substate_value, MappingContext, to_api_entity_address};
use crate::da::scan_result::*;
use crate::da::db::*;
use crate::da::processing_context::ProcessingContext;
use crate::da::processors::*;

mod scan_result;
mod db;
mod lru_like_dictionary;
mod processors;
mod processing_context;

// TODO #1: reduce .clone()ing
// TODO #2: reduce primitive variables usage
// TODO #3: use separate channels for scanning and DB persistence; possibly use even separate channel for read rocksdb + create missing definitions?
// TODO #4: eliminate all of those Box<dyn Error>
// TODO #5: actually does it even makes sense to process TX stream in batches if we stream it from the database? on a one hand batching eliminates tone of network round-trips of "find existing/most-recent" queries but maybe it's not worth it after all? batch saves are definitely valuable!

pub fn da_main(core_api_state: CoreApiState) -> Result<(), Box<dyn Error>> {
    let mut postgres_db = Client::connect("host=localhost user=db_dev_superuser password=db_dev_password dbname=rust_agg", NoTls)?;
    let sequences = DbSequences::new(&mut postgres_db)?;
    let ledger_tip = read_ledger_tip(&mut postgres_db);
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

        // TODO similarly to the Core API we'll fetch batches of 1000 TXs but this is definitely FAR from being most performant
        let bundles_iter = node_db.get_committed_transaction_bundle_iter(StateVersion::of(processing_context.ledger_tip as u64 + 1));
        let proofs_iter = Box::new(iter::empty());
        let transactions_and_proofs_iter = TransactionAndProofIterator::new(bundles_iter, proofs_iter);

        let mut tx_stream = vec![];
        for (bundle, _) in transactions_and_proofs_iter.take(limit) {
            // TODO obviously normally we'd operate on the bundle itself but as I want to keep the code similar to the original prototype we'd do a bit of copying

            let CommittedTransactionBundle {
                state_version,
                raw: _,
                receipt,
                identifiers: _,
            } = bundle;

            processing_context.ledger_tip = state_version.number() as i64;
            let mut changes = vec![];
            let substate_level_changes = receipt.on_ledger.state_changes.substate_level_changes;
            let context = MappingContext::new_for_transaction_stream(&core_api_state.network);

            // copied from the original Core API

            // Step 1 - First, build actions
            let mut changes_to_map = Vec::new();
            for (substate_reference, action) in substate_level_changes.iter() {
                let SubstateReference(node_id, partition_number, substate_key) = &substate_reference;
                let typed_substate_key =
                    create_typed_substate_key(&context, node_id, *partition_number, substate_key).map_err(|_| "blee 4")?;
                if !typed_substate_key.value_is_mappable() {
                    continue;
                }
                changes_to_map.push((substate_reference, typed_substate_key, action))
            }

            // Step 2 - Build supplementary lookups from the database
            // let state_mapping_lookups =
            //     StateMappingLookups::create_from_database(Some(&node_db), &changes_to_map)?;

            // Step 3 - Map the change actions
            for (substate_reference, typed_substate_key, action) in changes_to_map.into_iter() {
                let SubstateReference(node_id, _, _) = substate_reference;

                let db_substate = match action {
                    SubstateChangeAction::Create { new } => Some(new),
                    SubstateChangeAction::Update { previous: _, new } => Some(new),
                    SubstateChangeAction::Delete { .. } => None,
                };

                if let Some(db_substate) = db_substate {
                    let raw: &[u8] = db_substate;

                    if let TypedSubstateKey::MetadataModule(TypedMetadataModuleSubstateKey::MetadataEntryKey(metadata_module_key)) = &typed_substate_key {
                        let entity_address = to_api_entity_address(&context, &node_id).map_err(|_| "blee 6")?;
                        let _ = create_typed_substate_value(&typed_substate_key, &raw).map_err(|_| "blee 7")?;

                        // if let TypedSubstateValue::MetadataModule(TypedMetadataModuleSubstateValue::MetadataEntry(metadata_value_substate)) = typed_value {
                        //     let a = metadata_value_substate.into_value().unwrap();
                        //     let b = scrypto_encode(&a).unwrap();
                        //
                        //
                        // }

                        changes.push(Change {
                            entity_address,
                            metadata_key: metadata_module_key.clone(),
                            metadata_value: hex::encode(raw),
                        });
                    }
                }
            }

            tx_stream.push(Tx {
                state_version: state_version.number() as i64,
                changes,
            });
        }

        println!("[DA][MAN][{:?}] batch of {} prepared", processing_context.elapsed(), tx_stream.len());

        if tx_stream.len() == 0 {
            println!("[DA][MAN] waiting...");

            thread::sleep(Duration::from_millis(1000));

            continue;
        }

        // second loop
        proc_execute_processors(&mut processing_context, &sequences, &tx_stream)?;
        proc_store(&mut processing_context, &sequences)?;
        proc_fin(&mut processing_context)?;
    }

    println!("[DA][MAN] DONE");

    Ok(())
}

fn proc_execute_processors(pc: &mut ProcessingContext, seq: &DbSequences, tx_stream: &[Tx]) -> Result<(), Box<dyn Error>> {
    // STEP 1
    let existing_entities_by_address = &mut pc.existing_entities_by_address;
    let existing_entities = &mut pc.existing_entities;
    let scan_results = scan_tx_stream(&tx_stream);

    let mut entity_definitions_to_load = vec![];
    for (k, _) in &scan_results.entity_definitions {
        entity_definitions_to_load.push(k.0.clone())
    }

    // actually loads from the db
    for (_, ed) in existing_entity_definitions(&mut pc.db_conn, entity_definitions_to_load.as_slice())? {
        let ed = Rc::new(ed);
        existing_entities_by_address.put(ed.address.clone(), Rc::clone(&ed));
        existing_entities.put(ed.to_lookup(), Rc::clone(&ed));
    }

    // create entities increase
    let entity_increase = todo_rename_create_missing_entities(existing_entities_by_address, existing_entities, &seq, &scan_results);
    pc.increases.push(Box::new(entity_increase));

    // STEP 2
    let most_recent_entries = &mut pc.most_recent_metadata_entry_hisotry;
    let most_recent_aggregates = &mut pc.most_recent_metadata_aggregate_history;

    let mut a_to_load = vec![];
    let mut e_to_load = vec![];
    for x in scan_results.metadata_aggregate_history {
        if let Some(ee) = existing_entities_by_address.get(&x.0) {
            a_to_load.push(ee.id);
        }
    }
    for x in scan_results.metadata_entry_history {
        if let Some(ee) = existing_entities_by_address.get(&x.0) {
            e_to_load.push(DbMetadataEntryHistoryLookup {
                entity_id: ee.id,
                key: x.1.clone().as_bytes().to_vec(),
            });
        }
    }
    for (k, v) in most_recent_metadata_aggregate_history(&mut pc.db_conn, a_to_load.as_slice())? {
        most_recent_aggregates.put(k, Rc::new(RefCell::new(v)));
    }
    for (k, v) in most_recent_metadata_entry_history(&mut pc.db_conn, e_to_load.as_slice())? {
        most_recent_entries.put(k, Rc::new(v));
    }

    let mut ltp = LedgerTransactionProcessor::new();
    let mut mp = MetadataProcessor::new();
    let mut new_tip = pc.ledger_tip;

    for tx in tx_stream {
        new_tip = tx.state_version;

        ltp.process_tx(tx, seq)?;

        for change in &tx.changes {
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
    let ee = pc.existing_entities_by_address.evict(|x| x.from_state_version < pc.ledger_tip);
    let mra = pc.most_recent_metadata_aggregate_history.evict(|x| x.borrow().from_state_version < pc.ledger_tip);
    let mre = pc.most_recent_metadata_entry_hisotry.evict(|x| x.from_state_version < pc.ledger_tip);
    let evicted = ee + mra + mre;

    println!("[DA][FIN][{:?}] cleared {} out of {} LRU-like entries", pc.elapsed(), evicted, total);

    Ok(())
}
