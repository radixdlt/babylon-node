use crate::engine_state_api::*;

use crate::engine_prelude::*;

use std::ops::Deref;

pub(crate) async fn handle_object_role_assignment(
    state: State<EngineStateApiState>,
    Json(request): Json<models::ObjectRoleAssignmentRequest>,
) -> Result<Json<models::ObjectRoleAssignmentResponse>, ResponseError> {
    let mapping_context = MappingContext::new(&state.network);
    let extraction_context = ExtractionContext::new(&state.network);

    let node_id = extract_address_as_node_id(&extraction_context, &request.entity_address)
        .map_err(|err| err.into_response_error("entity_address"))?;

    let database = state.state_manager.database.snapshot();
    let loader = ObjectRoleAssignmentLoader::new(database.deref());

    let ObjectRoleAssignment {
        owner_role_entry,
        main_module_roles,
        attached_modules,
    } = loader.load_role_assignment(&node_id)?;

    let header = read_current_ledger_header(database.deref());

    Ok(Json(models::ObjectRoleAssignmentResponse {
        at_ledger_state: Box::new(to_api_ledger_state_summary(&mapping_context, &header)?),
        owner: Box::new(to_api_owner_role_entry(&mapping_context, owner_role_entry)?),
        main_module_roles: to_api_module_roles(&mapping_context, main_module_roles)?,
        attached_modules: attached_modules
            .into_iter()
            .map(|(attached_module_id, module_roles)| {
                Ok::<_, MappingError>(models::ObjectRoleAssignmentResponseAttachedModulesInner {
                    attached_module_id: to_api_attached_module_id(&attached_module_id),
                    roles: to_api_module_roles(&mapping_context, module_roles)?,
                })
            })
            .collect::<Result<Vec<_>, _>>()?,
    }))
}

fn to_api_owner_role_entry(
    context: &MappingContext,
    owner_role_entry: OwnerRoleEntry,
) -> Result<models::OwnerRoleEntry, MappingError> {
    Ok(models::OwnerRoleEntry {
        rule: Some(to_api_access_rule(context, &owner_role_entry.rule)?),
        updater: match owner_role_entry.updater {
            OwnerRoleUpdater::None => models::OwnerRoleUpdater::None,
            OwnerRoleUpdater::Owner => models::OwnerRoleUpdater::Owner,
            OwnerRoleUpdater::Object => models::OwnerRoleUpdater::Object,
        },
    })
}

fn to_api_module_roles(
    context: &MappingContext,
    module_roles: ModuleRoles,
) -> Result<Vec<models::RoleAssignmentEntry>, MappingError> {
    module_roles
        .into_iter()
        .map(|(RoleKey { key }, assignment)| {
            Ok(models::RoleAssignmentEntry {
                key,
                assignment: Some(to_api_assignment(context, assignment)?),
            })
        })
        .collect::<Result<Vec<_>, _>>()
}

fn to_api_assignment(
    context: &MappingContext,
    assignment: Assignment,
) -> Result<models::Assignment, MappingError> {
    Ok(match assignment {
        Assignment::Owner => models::Assignment::OwnerAssignment {},
        Assignment::Explicit(rule) => models::Assignment::ExplicitAssignment {
            rule: Box::new(to_api_access_rule(context, &rule)?),
        },
    })
}
