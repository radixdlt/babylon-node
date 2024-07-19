use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;
use postgres::Transaction;
use radix_engine::object_modules::role_assignment::{RoleAssignmentAccessRuleEntryPayload, RoleAssignmentField, RoleAssignmentOwnerFieldPayload};
use radix_engine::system::system_substates::{FieldSubstate, FieldSubstateV1, KeyValueEntrySubstate, KeyValueEntrySubstateV1, LockStatus};
use radix_engine_interface::prelude::ModuleRoleKey;
use radix_substate_store_queries::typed_substate_layout::{TypedRoleAssignmentModuleSubstateValue, TypedRoleAssignmentSubstateKey, TypedSubstateKey, TypedSubstateValue};
use crate::da::{SubstateChange, Tx, TypedChangeAction};
use crate::da::db::*;
use crate::da::lru_like_dictionary::LruLikeDictionary;
use crate::da::processors::DbIncrease;

pub struct RoleAssignmentProcessor {
    entries_to_add: Vec<Rc<DbRoleAssignmentEntryHistory>>,
    aggregates_to_add: Vec<Rc<RefCell<DbRoleAssignmentAggregateHistory>>>,
}

enum ChangeData<'a> {
    OwnerRole { key: &'a RoleAssignmentField, new_value: &'a FieldSubstateV1<RoleAssignmentOwnerFieldPayload> },
    Rule { key: &'a ModuleRoleKey, new_value: &'a KeyValueEntrySubstateV1<RoleAssignmentAccessRuleEntryPayload> },
}

impl RoleAssignmentProcessor {
    pub fn new() -> Self {
        Self {
            entries_to_add: vec![],
            aggregates_to_add: vec![],
        }
    }

    pub fn process_change(
        &mut self,
        change: &SubstateChange,
        tx: &Tx,
        sequences: &DbSequences,
        existing_entities: &mut LruLikeDictionary<String, Rc<DbEntityDefinition>>,
        most_recent_entries: &mut LruLikeDictionary<DbRoleAssignmentEntryHistoryLookup, Rc<DbRoleAssignmentEntryHistory>>,
        most_recent_aggregates: &mut LruLikeDictionary<DbRoleAssignmentAggregateHistoryLookup, Rc<RefCell<DbRoleAssignmentAggregateHistory>>>
    ) -> Result<(), Box<dyn Error>> {
        // TODO support for owner role

        let Some(ChangeData::Rule { key: data_key, new_value: data_new_value }) = self.should_process(change) else {
            return Ok(());
        };

        let entity = existing_entities.get(&change.node_address.clone()).expect("ble, must exist");
        let previous_aggregate = most_recent_aggregates.get(&(entity.as_ref().into()));
        let tmp_aggregate;

        let mut aggregate = match previous_aggregate {
            Some(&ref e) if e.borrow().from_state_version == tx.state_version.number() as i64 => {
                e.borrow_mut()
            }
            _ => {
                let new_aggregate = DbRoleAssignmentAggregateHistory {
                    id: sequences.next_role_assignment_aggregate_history_id(),
                    from_state_version: tx.state_version.number() as i64,
                    entity_id: entity.id,
                    owner_role_id: -1, // TODO implement
                    entry_ids: match previous_aggregate {
                        None => vec![],
                        Some(&ref e) => e.borrow().entry_ids.clone(),
                    },
                };

                tmp_aggregate = Rc::new(RefCell::new(new_aggregate));
                self.aggregates_to_add.push(Rc::clone(&tmp_aggregate));

                most_recent_aggregates.put(entity.as_ref().into(), Rc::clone(&tmp_aggregate));

                tmp_aggregate.borrow_mut()
            }
        };

        let value = data_new_value.value.as_ref();

        let entry_lookup = DbRoleAssignmentEntryHistoryLookup { entity_id: entity.id, key_role: data_key.key.key.clone(), key_module: format!("{:?}", data_key.module) };
        let previous_entry = most_recent_entries.get(&entry_lookup);
        let previous_position = if let Some(previous_entry) = previous_entry {
            aggregate.entry_ids.iter().position(|&x| x == previous_entry.id)
        } else {
            None
        };

        let new_entry_id = sequences.next_role_assignment_entry_history_id();
        let new_entry = DbRoleAssignmentEntryHistory {
            id: new_entry_id,
            from_state_version: tx.state_version.number() as i64,
            entity_id: entity.id,
            key_role: entry_lookup.key_role.clone(),
            key_module: entry_lookup.key_module.clone(),
            value: value.map(|_x| vec![1, 2, 3]), // TODO implement
            is_deleted: value.is_none(),
            is_locked: matches!(data_new_value.lock_status, LockStatus::Locked),
        };

        if let Some(previous_position) = previous_position {
            aggregate.entry_ids.remove(previous_position);
        }

        aggregate.entry_ids.insert(0, new_entry_id);

        let tmp_entry = Rc::new(new_entry);
        self.entries_to_add.push(Rc::clone(&tmp_entry));
        most_recent_entries.put(entry_lookup, tmp_entry);

        Ok(())
    }

    fn should_process<'a, 'b>(&'a self, change: &'b SubstateChange) -> Option<ChangeData<'b>> {
        if let TypedSubstateKey::RoleAssignmentModule(substate_key) = &change.key {
            let TypedChangeAction::Upsert { new, previous: _ } = &change.action else {
                panic!("impossible! ra1");
            };

            return match substate_key {
                TypedRoleAssignmentSubstateKey::RoleAssignmentField(role_assignment) => {
                    let TypedSubstateValue::RoleAssignmentModule(TypedRoleAssignmentModuleSubstateValue::OwnerRole(FieldSubstate::V1(new_value))) = new else {
                        panic!("impossible! ra2");
                    };

                    Some(ChangeData::OwnerRole {
                        key: role_assignment,
                        new_value,
                    })
                }
                TypedRoleAssignmentSubstateKey::Rule(module_role_key) => {
                    let TypedSubstateValue::RoleAssignmentModule(TypedRoleAssignmentModuleSubstateValue::Rule(KeyValueEntrySubstate::V1(new_value))) = new else {
                        panic!("impossible! ra3");
                    };

                    Some(ChangeData::Rule {
                        key: module_role_key,
                        new_value,
                    })
                }
            }
        }

        None
    }
}

impl DbIncrease for RoleAssignmentProcessor {
    fn save_changes(&self, client: &mut Transaction) -> Result<u64, Box<dyn Error>> {
        let mut cnt = 0;

        cnt += persist_role_assignment_entry_history(client, &self.entries_to_add)?;
        cnt += persist_role_assignment_aggregate_history(client, &self.aggregates_to_add)?;

        Ok(cnt)
    }
}
