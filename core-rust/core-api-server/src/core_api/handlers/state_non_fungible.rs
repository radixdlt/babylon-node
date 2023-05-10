use radix_engine::blueprints::resource::*;
use radix_engine::types::*;
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

    let database = state.database.read();

    let id_type_substate: NonFungibleResourceManagerIdTypeSubstate =
        read_mandatory_main_field_substate(
            database.deref(),
            resource_address.as_node_id(),
            &NonFungibleResourceManagerField::IdType.into(),
        )?;

    let non_fungible_id =
        extract_non_fungible_id_from_simple_representation(&request.non_fungible_id)
            .map_err(|err| err.into_response_error("non_fungible_id"))?;

    if non_fungible_id.id_type() != id_type_substate {
        return Err(ExtractionError::WrongNonFungibleIdType {
            expected: id_type_substate,
            actual: non_fungible_id.id_type(),
        }
        .into_response_error("non_fungible_id"));
    }

    let non_fungible = read_optional_collection_substate::<Option<ScryptoRawValue>>(
        database.deref(),
        resource_address.as_node_id(),
        NON_FUNGIBLE_RESOURCE_MANAGER_DATA_STORE,
        &SubstateKey::Map(non_fungible_id.to_key()),
    )
    .flatten()
    .ok_or_else(|| {
        not_found_error("The given non_fungible_id doesn't exist under that resource address")
    })?;

    Ok(StateNonFungibleResponse {
        non_fungible: Some(to_api_non_fungible_resource_manager_data_substate(
            &mapping_context,
            &scrypto_encode(&non_fungible).unwrap(),
        )?),
    })
    .map(Json)
}
