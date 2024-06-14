use std::collections::HashMap;
use std::error::Error;
use postgres::Client;
use crate::da::db::*;

pub fn read_ledger_tip(postgres_db: &mut Client) -> Result<Option<i64>, Box<dyn Error>> {
    Ok(postgres_db.query_one("SELECT max(state_version) FROM ledger_transactions;", &[])?.get(0))
}

pub fn read_next_sequence_id(postgres_db: &mut Client, table_name: &str) -> Result<i64, Box<dyn Error>> {
    let sequence_id = format!("{}_id_seq", table_name);

    Ok(postgres_db.query_one("SELECT nextval($1::text);", &[&sequence_id])?.get(0))
}

pub fn existing_entity_definitions(postgres_db: &mut Client, lookup: &[String]) -> Result<HashMap<DbEntityDefinitionLookup, DbEntityDefinition>, Box<dyn Error>> {
    let mut res = HashMap::new();

    if lookup.len() > 0 {
        let rows = postgres_db.query(
            r"
            SELECT id, from_state_version, address
            FROM entity_definitions
            WHERE address = ANY($1)",
            &[&lookup])?;

        for row in rows {
            let key = DbEntityDefinitionLookup {
                id: row.get(0),
            };
            let value = DbEntityDefinition {
                id: row.get(0),
                from_state_version: row.get(1),
                address: row.get(2),
            };

            res.insert(key, value);
        }
    }

    Ok(res)
}

pub fn most_recent_metadata_entry_history(postgres_db: &mut Client, lookup: &[DbMetadataEntryHistoryLookup]) -> Result<HashMap<DbMetadataEntryHistoryLookup, DbMetadataEntryHistory>, Box<dyn Error>> {
    let mut res = HashMap::new();

    if lookup.len() > 0 {
        let mut entity_ids = vec![];
        let mut keys = vec![];

        for l in lookup.into_iter() {
            entity_ids.push(l.entity_id);
            keys.push(l.key.clone());
        }

        let rows = postgres_db.query(
            r"
            WITH variables (entity_id, key) AS (SELECT UNNEST($1::bigint[]), UNNEST($2::bytea[]))
            SELECT mr.id, mr.from_state_version, mr.entity_id, mr.key, mr.value
            FROM variables var
            INNER JOIN LATERAL (
                SELECT *
                FROM metadata_entry_history
                WHERE entity_id = var.entity_id AND key = var.key
                ORDER BY from_state_version DESC
                LIMIT 1
            ) mr ON true;",
            &[&entity_ids, &keys])?;

        for row in rows {
            let key = DbMetadataEntryHistoryLookup {
                entity_id: row.get(2),
                key: row.get(3),
            };
            let value = DbMetadataEntryHistory {
                id: row.get(0),
                from_state_version: row.get(1),
                entity_id: row.get(2),
                key: row.get(3),
                value: row.get(4),
            };

            res.insert(key, value);
        }
    }

    Ok(res)
}

pub fn most_recent_metadata_aggregate_history(postgres_db: &mut Client, lookup: &[i64]) -> Result<HashMap<DbMetadataAggregateHistoryLookup, DbMetadataAggregateHistory>, Box<dyn Error>> {
    let mut res = HashMap::new();

    if lookup.len() > 0 {
        let rows = postgres_db.query(
            r"
            WITH variables (entity_id) AS (SELECT UNNEST($1::bigint[]))
            SELECT mr.id, mr.from_state_version, mr.entity_id, mr.entry_ids
            FROM variables var
            INNER JOIN LATERAL (
                SELECT *
                FROM metadata_aggregate_history
                WHERE entity_id = var.entity_id
                ORDER BY from_state_version DESC
                LIMIT 1
            ) mr ON true;",
            &[&lookup])?;

        for row in rows {
            let key = DbMetadataAggregateHistoryLookup {
                id: row.get(0),
            };
            let value = DbMetadataAggregateHistory {
                id: row.get(0),
                from_state_version: row.get(1),
                entity_id: row.get(2),
                entry_ids: row.get(3),
            };

            res.insert(key, value);
        }
    }

    Ok(res)
}