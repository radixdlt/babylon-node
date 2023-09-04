use crate::core_api::*;

use radix_engine::types::*;
use radix_engine_queries::typed_substate_layout::*;
use state_manager::store::traits::QueryableProofStore;
use std::ops::Deref;

use radix_engine_common::types::EntityType;

enum ManagerByType {
    Fungible(
        FungibleResourceManagerDivisibilityFieldSubstate,
        Option<FungibleResourceManagerTotalSupplyFieldSubstate>,
    ),
    NonFungible(
        NonFungibleResourceManagerIdTypeFieldSubstate,
        Option<NonFungibleResourceManagerTotalSupplyFieldSubstate>,
        NonFungibleResourceManagerMutableFieldsFieldSubstate,
    ),
}

pub(crate) async fn handle_state_resource(
    state: State<CoreApiState>,
    Json(request): Json<models::StateResourceRequest>,
) -> Result<Json<models::StateResourceResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;
    let mapping_context = MappingContext::new(&state.network);
    let extraction_context = ExtractionContext::new(&state.network);

    let resource_address = extract_resource_address(&extraction_context, &request.resource_address)
        .map_err(|err| err.into_response_error("resource_address"))?;

    let database = state.state_manager.database.read();

    let resource_node_id = resource_address.as_node_id();
    let is_fungible =
        resource_node_id.entity_type() == Some(EntityType::GlobalFungibleResourceManager);
    let manager = if is_fungible {
        ManagerByType::Fungible(
            read_optional_main_field_substate(
                database.deref(),
                resource_node_id,
                &FungibleResourceManagerField::Divisibility.into(),
            )
            .ok_or_else(|| not_found_error("Resource not found".to_string()))?,
            read_optional_main_field_substate(
                database.deref(),
                resource_node_id,
                &FungibleResourceManagerField::TotalSupply.into(),
            ),
        )
    } else {
        ManagerByType::NonFungible(
            read_optional_main_field_substate(
                database.deref(),
                resource_node_id,
                &NonFungibleResourceManagerField::IdType.into(),
            )
            .ok_or_else(|| not_found_error("Resource not found".to_string()))?,
            read_optional_main_field_substate(
                database.deref(),
                resource_node_id,
                &NonFungibleResourceManagerField::TotalSupply.into(),
            ),
            read_mandatory_main_field_substate(
                database.deref(),
                resource_node_id,
                &NonFungibleResourceManagerField::MutableFields.into(),
            )?,
        )
    };

    let owner_role_substate = read_mandatory_substate(
        database.deref(),
        resource_address.as_node_id(),
        RoleAssignmentPartitionOffset::Field.as_partition(ROLE_ASSIGNMENT_BASE_PARTITION),
        &RoleAssignmentField::Owner.into(),
    )?;

    let header = database
        .get_last_proof()
        .expect("proof for outputted state must exist")
        .ledger_header;

    Ok(models::StateResourceResponse {
        at_ledger_state: Box::new(to_api_ledger_state_summary(&mapping_context, &header)?),
        manager: Some(to_api_resource_manager(&mapping_context, &manager)?),
        owner_role: Some(to_api_owner_role_substate(
            &mapping_context,
            &owner_role_substate,
        )?),
    })
    .map(Json)
}

fn to_api_resource_manager(
    context: &MappingContext,
    manager: &ManagerByType,
) -> Result<models::StateResourceManager, MappingError> {
    Ok(match manager {
        ManagerByType::Fungible(divisibility, total_supply) => {
            models::StateResourceManager::StateFungibleResourceManager {
                divisibility: Box::new(to_api_fungible_resource_manager_divisibility_substate(
                    divisibility,
                )?),
                total_supply: total_supply
                    .as_ref()
                    .map(to_api_fungible_resource_manager_total_supply_substate)
                    .transpose()?
                    .map(Box::new),
            }
        }
        ManagerByType::NonFungible(id_type, total_supply, mutable_fields) => {
            models::StateResourceManager::StateNonFungibleResourceManager {
                id_type: Box::new(to_api_non_fungible_resource_manager_id_type_substate(
                    id_type,
                )?),
                total_supply: total_supply
                    .as_ref()
                    .map(to_api_non_fungible_resource_manager_total_supply_substate)
                    .transpose()?
                    .map(Box::new),
                mutable_fields: Box::new(
                    to_api_non_fungible_resource_manager_mutable_fields_substate(
                        context,
                        mutable_fields,
                    )?,
                ),
            }
        }
    })
}
