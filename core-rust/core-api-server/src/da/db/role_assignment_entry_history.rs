use std::collections::HashMap;
use std::error::Error;
use std::rc::Rc;
use postgres::binary_copy::BinaryCopyInWriter;
use postgres::{Client, Transaction};
use postgres::types::Type;

#[derive(Debug)]
pub struct DbRoleAssignmentEntryHistory {
    pub id: i64,
    pub from_state_version: i64,
    pub entity_id: i64,
    pub key_role: String,
    pub key_module: String,
    pub value: Option<Vec<u8>>,
    pub is_deleted: bool,
    pub is_locked: bool,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct DbRoleAssignmentEntryHistoryLookup {
    pub entity_id: i64,
    pub key_role: String,
    pub key_module: String,
}

impl From<&DbRoleAssignmentEntryHistory> for DbRoleAssignmentEntryHistoryLookup {
    fn from(value: &DbRoleAssignmentEntryHistory) -> Self {
        Self {
            entity_id: value.entity_id,
            key_role: value.key_role.clone(),
            key_module: value.key_module.clone(),
        }
    }
}

pub fn persist_role_assignment_entry_history(postgres_db: &mut Transaction, db_entities: &[Rc<DbRoleAssignmentEntryHistory>]) -> Result<u64, Box<dyn Error>> {
    if db_entities.len() == 0 {
        return Ok(0);
    }

    let sink = postgres_db.copy_in("COPY role_assignment_entry_history (id, from_state_version, entity_id, key_role, key_module, value, is_deleted, is_locked) FROM STDIN (FORMAT BINARY)")?;
    let mut writer = BinaryCopyInWriter::new(sink, &[Type::INT8, Type::INT8, Type::INT8, Type::TEXT, Type::TEXT, Type::BYTEA, Type::BOOL, Type::BOOL]);

    for e in db_entities {
        writer.write(&[&e.id, &e.from_state_version, &e.entity_id, &e.key_role, &e.key_module, &e.value, &e.is_deleted, &e.is_locked])?;
    }

    Ok(writer.finish()?)
}

pub fn most_recent_role_assignment_entry_history(postgres_db: &mut Client, lookup: &[DbRoleAssignmentEntryHistoryLookup]) -> Result<HashMap<DbRoleAssignmentEntryHistoryLookup, DbRoleAssignmentEntryHistory>, Box<dyn Error>> {
    let mut res = HashMap::new();

    if lookup.len() > 0 {
        let mut entity_ids = vec![];
        let mut key_roles = vec![];
        let mut key_modules = vec![];

        for l in lookup.into_iter() {
            entity_ids.push(l.entity_id);
            key_roles.push(l.key_role.clone());
            key_modules.push(l.key_module.clone());
        }

        let rows = postgres_db.query(
            r"
            WITH variables (entity_id, key_role, key_module) AS (SELECT UNNEST($1::bigint[]), UNNEST($2::text[]), UNNEST($3::text[]))
            SELECT mr.id, mr.from_state_version, mr.entity_id, mr.key_role, mr.key_module, mr.value, mr.is_deleted, mr.is_locked
            FROM variables var
            INNER JOIN LATERAL (
                SELECT *
                FROM role_assignment_entry_history
                WHERE entity_id = var.entity_id AND key_role = var.key_role AND key_module = var.key_module
                ORDER BY from_state_version DESC
                LIMIT 1
            ) mr ON true;",
            &[&entity_ids, &key_roles, &key_modules])?;

        for row in rows {
            let value = DbRoleAssignmentEntryHistory {
                id: row.get(0),
                from_state_version: row.get(1),
                entity_id: row.get(2),
                key_role: row.get(3),
                key_module: row.get(4),
                value: row.get(5),
                is_deleted: row.get(6),
                is_locked: row.get(7),
            };

            res.insert((&value).into(), value);
        }
    }

    Ok(res)
}