use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use postgres::{Client, NoTls};
use crate::core_api::DaState;
use crate::da::aggregate::*;
use crate::da::db::*;

mod db;
mod aggregate;
mod lru_like_dictionary;

// TODO #1: reduce .clone()ing
// TODO #2: reduce primitive variables usage
// TODO #3: use separate channels for scanning and DB persistence

pub fn da_main(inner_da_state: Arc<Mutex<DaState>>) {
    let mut postgres_db = Client::connect("host=localhost user=db_dev_superuser password=db_dev_password dbname=rust_agg", NoTls).unwrap();
    let mut sequences = DbSequences::new(&mut postgres_db);
    let ledger_tip = read_ledger_tip(&mut postgres_db);

    let mut fetch_context = FetchContext::new(ledger_tip);
    let mut processing_context = ProcessingContext::new(&mut postgres_db);

    let processors: Vec<fn(&mut ProcessingContext, &mut DbSequences, &[Event]) -> ()> = vec![
        proc_init,
        proc_scan,
        proc_execute_processors,
        proc_store,
        proc_fin,
    ];

    loop {
        let running = inner_da_state.lock().unwrap().should_run;

        if !running {
            break;
        }

        let event_stream = fetch_sample_event_stream(&mut fetch_context);

        for p in &processors {
            p(&mut processing_context, &mut sequences, &event_stream);
        }
    }
}

fn proc_init(_: &mut ProcessingContext, _: &mut DbSequences, event_stream: &[Event]) {
    println!("[DA][INT] about to process {} events", event_stream.len());
}

fn proc_scan(pc: &mut ProcessingContext, _: &mut DbSequences, event_stream: &[Event]) {
    let sr = scan_event_stream(event_stream);

    pc.scan_results.replace(sr);
}

fn proc_execute_processors(pc: &mut ProcessingContext, seq: &mut DbSequences, event_stream: &[Event]) {
    let existing_entities = &mut pc.existing_entities;
    let scan_results = pc.scan_results.take().unwrap();

    let mut ed_to_load = vec![];
    for (k, _) in &scan_results.entity_definitions {
        ed_to_load.push(k.0.clone())
    }
    for ed in existing_entity_definitions(&mut pc.db_conn, ed_to_load.as_slice()) {
        existing_entities.put(ed.address.clone(), Rc::new(ed));
    }

    let increase_a = process_event_stream_step_a(existing_entities, &seq, &scan_results);
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
    for (k, v) in most_recent_metadata_aggregate_history(&mut pc.db_conn, a_to_load.as_slice()) {
        most_recent_aggregates.put(k, Rc::new(RefCell::new(v)));
    }
    for (k, v) in most_recent_metadata_entry_history(&mut pc.db_conn, e_to_load.as_slice()) {
        most_recent_entries.put(k, Rc::new(v));
    }

    let increase_b = process_event_stream_step_b(existing_entities, most_recent_entries, most_recent_aggregates, &seq, event_stream);
    let diff = increase_b.last_state_version - pc.min_state_version.unwrap_or(0);

    pc.increase_b.replace(increase_b);

    println!("[DA][EXE] processed {} events", diff);
}

fn proc_store(pc: &mut ProcessingContext, seq: &mut DbSequences, _: &[Event]) {
    let increase_a = pc.increase_a.take().unwrap();
    let increase_b = pc.increase_b.take().unwrap();

    println!(
        "[DA][STR] about to push to database: entity_definitions={}, ledger_transactions={}, metadata_aggregate={}, metadata_entry={}",
        increase_a.entity_definitions.len(),
        increase_b.ledger_transactions.len(),
        increase_b.metadata_aggregate_history.len(),
        increase_b.metadata_entry_history.len());

    let mut db_tx = pc.db_conn.transaction().unwrap();

    persist_entity_definitions(&mut db_tx, &increase_a.entity_definitions);
    persist_ledger_transactions(&mut db_tx, &increase_b.ledger_transactions);
    persist_metadata_entry_history(&mut db_tx, &increase_b.metadata_entry_history);
    persist_metadata_aggregate_history(&mut db_tx, &increase_b.metadata_aggregate_history);
    seq.persist(&mut db_tx);
    db_tx.commit().unwrap();

    pc.min_state_version.replace(increase_b.last_state_version);
}

fn proc_fin(pc: &mut ProcessingContext, _: &mut DbSequences, _: &[Event]) {
    if let Some(min_state_version) = pc.min_state_version {
        let ee = pc.existing_entities.evict(|x| x.from_state_version < min_state_version);
        let mra = pc.most_recent_aggregates.evict(|x| x.borrow().from_state_version < min_state_version);
        let mre = pc.most_recent_entries.evict(|x| x.from_state_version < min_state_version);

        println!("[DA][FIN] cleared {} LRU existing_entities, {} LRU most_recent_aggregates, {} LRU most_recent_entries", ee, mra, mre);
    }
}
