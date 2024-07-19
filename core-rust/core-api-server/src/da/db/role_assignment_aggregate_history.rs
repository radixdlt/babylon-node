use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::rc::Rc;
use postgres::binary_copy::BinaryCopyInWriter;
use postgres::{Client, Transaction};
use postgres::types::Type;
use crate::da::db::DbEntityDefinition;

#[derive(Debug)]
pub struct DbRoleAssignmentAggregateHistory {
    pub id: i64,
    pub from_state_version: i64,
    pub entity_id: i64,
    pub owner_role_id: i64,
    pub entry_ids: Vec<i64>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct DbRoleAssignmentAggregateHistoryLookup {
    pub entity_id: i64,
}

impl From<&DbRoleAssignmentAggregateHistory> for DbRoleAssignmentAggregateHistoryLookup {
    fn from(value: &DbRoleAssignmentAggregateHistory) -> Self {
        Self {
            entity_id: value.entity_id,
        }
    }
}

impl From<&DbEntityDefinition> for DbRoleAssignmentAggregateHistoryLookup {
    fn from(value: &DbEntityDefinition) -> Self {
        Self {
            entity_id: value.id,
        }
    }
}

pub fn persist_role_assignment_aggregate_history(postgres_db: &mut Transaction, db_entities: &[Rc<RefCell<DbRoleAssignmentAggregateHistory>>]) -> Result<u64, Box<dyn Error>> {
    if db_entities.len() == 0 {
        return Ok(0);
    }

    let sink = postgres_db.copy_in("COPY role_assignment_aggregate_history (id, from_state_version, entity_id, owner_role_id, entry_ids) FROM STDIN (FORMAT BINARY)")?;
    let mut writer = BinaryCopyInWriter::new(sink, &[Type::INT8, Type::INT8, Type::INT8, Type::INT8, Type::INT8_ARRAY]);

    for e in db_entities {
        let e = e.borrow();

        writer.write(&[&e.id, &e.from_state_version, &e.entity_id, &e.owner_role_id, &e.entry_ids])?;
    }

    Ok(writer.finish()?)
}

pub fn most_recent_role_assignment_aggregate_history(postgres_db: &mut Client, lookup: &[DbRoleAssignmentAggregateHistoryLookup]) -> Result<HashMap<DbRoleAssignmentAggregateHistoryLookup, DbRoleAssignmentAggregateHistory>, Box<dyn Error>> {
    let mut res = HashMap::new();

    if lookup.len() > 0 {
        let mut entity_ids = vec![];

        for l in lookup.into_iter() {
            entity_ids.push(l.entity_id);
        }

        let rows = postgres_db.query(
            r"
            WITH variables (entity_id) AS (SELECT UNNEST($1::bigint[]))
            SELECT mr.id, mr.from_state_version, mr.entity_id, mr.owner_role_id, mr.entry_ids
            FROM variables var
            INNER JOIN LATERAL (
                SELECT *
                FROM role_assignment_aggregate_history
                WHERE entity_id = var.entity_id
                ORDER BY from_state_version DESC
                LIMIT 1
            ) mr ON true;",
            &[&entity_ids])?;

        for row in rows {
            let value = DbRoleAssignmentAggregateHistory {
                id: row.get(0),
                from_state_version: row.get(1),
                entity_id: row.get(2),
                owner_role_id: row.get(3),
                entry_ids: row.get(4),
            };

            res.insert((&value).into(), value);
        }
    }

    Ok(res)
}