use crate::core_api::*;

use radix_engine::types::*;
use radix_engine_queries::typed_substate_layout::*;
use std::ops::Deref;

use radix_engine_common::types::EntityType;

enum ManagerByType {
    Fungible(
        FungibleResourceManagerDivisibilitySubstate,
        Option<FungibleResourceManagerTotalSupplySubstate>,
    ),
    NonFungible(
        NonFungibleResourceManagerIdTypeSubstate,
        Option<NonFungibleResourceManagerTotalSupplySubstate>,
        NonFungibleResourceManagerMutableFieldsSubstate,
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

    let database = state.database.read();

    let resource_node_id = resource_address.as_node_id();
    let is_fungible =
        resource_node_id.entity_type() == Some(EntityType::GlobalFungibleResourceManager);
    let manager = if is_fungible {
        ManagerByType::Fungible(
            read_mandatory_main_field_substate(
                database.deref(),
                resource_node_id,
                &FungibleResourceManagerField::Divisibility.into(),
            )?,
            read_optional_main_field_substate(
                database.deref(),
                resource_node_id,
                &FungibleResourceManagerField::TotalSupply.into(),
            ),
        )
    } else {
        ManagerByType::NonFungible(
            read_mandatory_main_field_substate(
                database.deref(),
                resource_node_id,
                &NonFungibleResourceManagerField::IdType.into(),
            )?,
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

    let owner_role_substate: OwnerRoleSubstate = read_mandatory_substate(
        database.deref(),
        resource_address.as_node_id(),
        ACCESS_RULES_FIELDS_PARTITION,
        &AccessRulesField::OwnerRole.into(),
    )?;

    Ok(models::StateResourceResponse {
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
        ManagerByType::Fungible(divisiility, total_supply) => {
            models::StateResourceManager::StateFungibleResourceManager {
                divisibility: Box::new(to_api_fungible_resource_manager_divisibility_substate(
                    divisiility,
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
                    .map(to_api_fungible_resource_manager_total_supply_substate)
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
