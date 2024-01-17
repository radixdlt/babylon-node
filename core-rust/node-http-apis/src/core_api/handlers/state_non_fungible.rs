use radix_engine::blueprints::resource::*;
use radix_engine::types::*;
use radix_engine_queries::typed_substate_layout::{TypedMainModuleSubstateKey, TypedSubstateKey};

use std::ops::Deref;

use crate::core_api::*;
use radix_engine_common::types::SubstateKey;

use crate::core_api::models::StateNonFungibleResponse;

pub(crate) async fn handle_state_non_fungible(
    state: State<CoreApiState>,
    Json(request): Json<models::StateNonFungibleRequest>,
) -> Result<Json<StateNonFungibleResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;

    let mapping_context = MappingContext::new(&state.network);
    let extraction_context = ExtractionContext::new(&state.network);

    let resource_address = extract_resource_address(&extraction_context, &request.resource_address)
        .map_err(|err| err.into_response_error("resource_address"))?;

    if Some(EntityType::GlobalNonFungibleResourceManager)
        != resource_address.as_node_id().entity_type()
    {
        return Err(client_error("Resource is not a non-fungible resource"));
    }

    let database = state.state_manager.database.read_current();

    let id_type =
        read_optional_main_field_substate::<NonFungibleResourceManagerIdTypeFieldPayload>(
            database.deref(),
            resource_address.as_node_id(),
            &NonFungibleResourceManagerField::IdType.into(),
        )
        .ok_or_else(|| not_found_error("Resource not found".to_string()))?
        .into_payload()
        .into_latest();

    let non_fungible_id =
        extract_non_fungible_id_from_simple_representation(&request.non_fungible_id)
            .map_err(|err| err.into_response_error("non_fungible_id"))?;

    if non_fungible_id.id_type() != id_type {
        return Err(ExtractionError::WrongNonFungibleIdType {
            expected: id_type,
            actual: non_fungible_id.id_type(),
        }
        .into_response_error("non_fungible_id"));
    }

    let substate = read_optional_collection_substate::<NonFungibleResourceManagerDataEntryPayload>(
        database.deref(),
        resource_address.as_node_id(),
        NonFungibleResourceManagerCollection::DataKeyValue.collection_index(),
        &SubstateKey::Map(non_fungible_id.to_key()),
    )
    .ok_or_else(|| {
        not_found_error("The given non_fungible_id doesn't exist under that resource address")
    })?;

    let header = read_current_ledger_header(database.deref());

    Ok(Json(StateNonFungibleResponse {
        at_ledger_state: Box::new(to_api_ledger_state_summary(&mapping_context, &header)?),
        non_fungible: Some(to_api_non_fungible_resource_manager_data_substate(
            &mapping_context,
            &TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::NonFungibleResourceManager(
                NonFungibleResourceManagerTypedSubstateKey::DataKeyValueEntry(
                    non_fungible_id.into(),
                ),
            )),
            &substate,
        )?),
    }))
}
