use std::cell::RefCell;
use std::error::Error;
use std::iter;
use std::rc::Rc;
use std::time::Instant;

use postgres::{Client, NoTls};
use radix_substate_store_queries::typed_substate_layout::{TypedMetadataModuleSubstateKey, TypedSubstateKey};

use state_manager::{StateVersion, SubstateChangeAction, SubstateReference};
use state_manager::traits::*;

use crate::core_api::{CoreApiState, create_typed_substate_key, create_typed_substate_value, MappingContext, to_api_entity_address};
use crate::da::aggregate::*;
use crate::da::db::*;

mod db;
mod aggregate;
mod lru_like_dictionary;

// TODO #1: reduce .clone()ing
// TODO #2: reduce primitive variables usage
// TODO #3: use separate channels for scanning and DB persistence; possibly use even separate channel for read rocksdb + create missing definitions?
// TODO #4: eliminate all of those Box<dyn Error>
// TODO #5: actually does it even makes sense to process TX stream in batches if we stream it from the database? on a one hand batching eliminates tone of network round-trips of "find existing/most-recent" queries but maybe it's not worth it after all? batch saves are definitely valuable!
// TODO #6: add from/into for trivial type mapping (e.g. lookup types)

pub fn da_main(core_api_state: CoreApiState) -> Result<(), Box<dyn Error>> {
    let mut postgres_db = Client::connect("host=localhost user=db_dev_superuser password=db_dev_password dbname=rust_agg", NoTls)?;
    let mut sequences = DbSequences::new(&mut postgres_db)?;
    let ledger_tip = read_ledger_tip(&mut postgres_db);

    let mut processing_context = ProcessingContext::new(&mut postgres_db);

    let processors: Vec<fn(&mut ProcessingContext, &mut DbSequences, &[Tx]) -> Result<(), Box<dyn Error>>> = vec![
        proc_init,
        proc_scan,
        proc_execute_processors,
        proc_store,
        proc_fin,
    ];

    let da_state = core_api_state.da_state;

    // TODO not sure if we can use direct access or should we use the .snapshot()
    let node_db = core_api_state.state_manager.database.access_direct();
    let mut fetch_state_version = ledger_tip?.unwrap_or(0) as u64;
    let limit = 1000;

    loop {
        let running = da_state.lock().unwrap().should_run;

        if !running {
            break;
        }

        processing_context.stopwatch.replace(Instant::now());

        // TODO similarly to the Core API we'll fetch batches of 1000 TXs but this is definitely FAR from being most performant
        let bundles_iter = node_db.get_committed_transaction_bundle_iter(StateVersion::of(fetch_state_version + 1));
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

            fetch_state_version = state_version.number();
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
                state_version: fetch_state_version as i64,
                changes,
            });
        }

        println!("[DA][MAN][{:?}] batch prepared", processing_context.elapsed());

        for p in &processors {
            p(&mut processing_context, &mut sequences, &tx_stream)?;
        }
    }

    println!("[DA] DONE");

    Ok(())
}

fn proc_init(_: &mut ProcessingContext, _: &mut DbSequences, _: &[Tx]) -> Result<(), Box<dyn Error>> {
    Ok(())
}

fn proc_scan(pc: &mut ProcessingContext, _: &mut DbSequences, tx_stream: &[Tx]) -> Result<(), Box<dyn Error>> {
    let sr = scan_tx_stream(tx_stream);

    pc.scan_results.replace(sr);

    Ok(())
}

fn proc_execute_processors(pc: &mut ProcessingContext, seq: &mut DbSequences, tx_stream: &[Tx]) -> Result<(), Box<dyn Error>> {
    let existing_entities = &mut pc.existing_entities;
    let scan_results = pc.scan_results.take().ok_or("blee 5")?;

    let mut ed_to_load = vec![];
    for (k, _) in &scan_results.entity_definitions {
        ed_to_load.push(k.0.clone())
    }
    for (_, ed) in existing_entity_definitions(&mut pc.db_conn, ed_to_load.as_slice())? {
        existing_entities.put(ed.address.clone(), Rc::new(ed));
    }

    let increase_a = process_tx_stream_step_a(existing_entities, &seq, &scan_results);
    pc.increase_a.replace(increase_a);

    let most_recent_entries = &mut pc.most_recent_entries;
    let most_recent_aggregates = &mut pc.most_recent_aggregates;

    let mut a_to_load = vec![];
    let mut e_to_load = vec![];
    for x in scan_results.metadata_aggregate_history {
        if let Some(ee) = existing_entities.get(&x.0) {
            a_to_load.push(ee.id);
        }
    }
    for x in scan_results.metadata_entry_history {
        if let Some(ee) = existing_entities.get(&x.0) {
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

    let increase_b = process_tx_stream_step_b(existing_entities, most_recent_entries, most_recent_aggregates, &seq, tx_stream);
    let diff = increase_b.last_state_version - pc.min_state_version.unwrap_or(0);

    pc.increase_b.replace(increase_b);

    println!("[DA][EXE][{:?}] processed {} TXs", pc.elapsed(), diff);

    Ok(())
}

fn proc_store(pc: &mut ProcessingContext, seq: &mut DbSequences, _: &[Tx]) -> Result<(), Box<dyn Error>> {
    let increase_a = pc.increase_a.take().ok_or("blee a")?;
    let increase_b = pc.increase_b.take().ok_or("blee b")?;

    let mut db_tx = pc.db_conn.transaction()?;
    let mut cnt = 0;

    cnt += persist_entity_definitions(&mut db_tx, &increase_a.entity_definitions)?;
    cnt += persist_ledger_transactions(&mut db_tx, &increase_b.ledger_transactions)?;
    cnt += persist_metadata_entry_history(&mut db_tx, &increase_b.metadata_entry_history)?;
    cnt += persist_metadata_aggregate_history(&mut db_tx, &increase_b.metadata_aggregate_history)?;
    cnt += seq.persist(&mut db_tx)?;

    db_tx.commit()?;

    pc.min_state_version.replace(increase_b.last_state_version);

    println!("[DA][STR][{:?}] pushed {} new entities", pc.elapsed(), cnt);

    Ok(())
}

fn proc_fin(pc: &mut ProcessingContext, _: &mut DbSequences, _: &[Tx]) -> Result<(), Box<dyn Error>> {
    if let Some(min_state_version) = pc.min_state_version {
        let total = pc.existing_entities.len() + pc.most_recent_aggregates.len() + pc.existing_entities.len();
        let ee = pc.existing_entities.evict(|x| x.from_state_version < min_state_version);
        let mra = pc.most_recent_aggregates.evict(|x| x.borrow().from_state_version < min_state_version);
        let mre = pc.most_recent_entries.evict(|x| x.from_state_version < min_state_version);

        println!("[DA][FIN][{:?}] cleared {} out of {} LRU-like entries", pc.elapsed(), total, ee + mra + mre);
    }

    Ok(())
}
