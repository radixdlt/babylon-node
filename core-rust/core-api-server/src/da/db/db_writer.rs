use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use postgres::binary_copy::BinaryCopyInWriter;
use postgres::Transaction;
use postgres::types::{ToSql, Type};
use crate::da::db::*;

pub fn persist_ledger_transactions(postgres_db: &mut Transaction, db_entities: &[Rc<DbLedgerTransaction>]) -> u64 {
    if db_entities.len() == 0 {
        return 0;
    }

    let sink = postgres_db.copy_in("COPY ledger_transactions (state_version, created_at) FROM STDIN (FORMAT BINARY)").unwrap();
    let mut writer = BinaryCopyInWriter::new(sink, &[Type::INT8, Type::TIMESTAMPTZ]);

    for e in db_entities {
        writer.write(&[&e.state_version, &e.created_at]).unwrap();
    }

    writer.finish().unwrap()
}

pub fn persist_entity_definitions(postgres_db: &mut Transaction, db_entities: &[Rc<DbEntityDefinition>]) -> u64 {
    if db_entities.len() == 0 {
        return 0;
    }

    let sink = postgres_db.copy_in("COPY entity_definitions (id, from_state_version, address) FROM STDIN (FORMAT BINARY)").unwrap();
    let mut writer = BinaryCopyInWriter::new(sink, &[Type::INT8, Type::INT8, Type::TEXT]);

    for e in db_entities {
        writer.write(&[&e.id, &e.from_state_version, &e.address]).unwrap();
    }

    writer.finish().unwrap()
}

pub fn persist_metadata_entry_history(postgres_db: &mut Transaction, db_entities: &[Rc<DbMetadataEntryHistory>]) -> u64 {
    if db_entities.len() == 0 {
        return 0;
    }

    let sink = postgres_db.copy_in("COPY metadata_entry_history (id, from_state_version, entity_id, key, value) FROM STDIN (FORMAT BINARY)").unwrap();
    let mut writer = BinaryCopyInWriter::new(sink, &[Type::INT8, Type::INT8, Type::INT8, Type::BYTEA, Type::BYTEA]);

    for e in db_entities {
        writer.write(&[&e.id, &e.from_state_version, &e.entity_id, &e.key, &e.value]).unwrap();
    }

    writer.finish().unwrap()
}

pub fn persist_metadata_aggregate_history(postgres_db: &mut Transaction, db_entities: &[Rc<RefCell<DbMetadataAggregateHistory>>]) -> u64 {
    if db_entities.len() == 0 {
        return 0;
    }

    let sink = postgres_db.copy_in("COPY metadata_aggregate_history (id, from_state_version, entity_id, entry_ids) FROM STDIN (FORMAT BINARY)").unwrap();
    let mut writer = BinaryCopyInWriter::new(sink, &[Type::INT8, Type::INT8, Type::INT8, Type::INT8_ARRAY]);

    for e in db_entities {
        let e = e.borrow();

        writer.write(&[&e.id, &e.from_state_version, &e.entity_id, &e.entry_ids]).unwrap();
    }

    writer.finish().unwrap()
}

// TODO ugh... :(
pub fn persist_sequences(postgres_db: &mut Transaction, sequences: HashMap<&str, i64>) {
    let mut c = 1;
    let mut query = String::from("SELECT ");
    let mut parameters: Vec<&(dyn ToSql + Sync)> = vec![];

    for (k, v) in &sequences {
        query.push_str(format!("setval('{}_id_seq', ${}), ", k, c).as_str());
        parameters.push(v);
        c += 1;
    }

    query.push_str("1");

    postgres_db.execute(query.as_str(), &parameters.as_slice()).unwrap();
}