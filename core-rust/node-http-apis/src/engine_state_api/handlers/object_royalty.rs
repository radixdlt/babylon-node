use crate::engine_state_api::*;

use crate::engine_prelude::*;

use state_manager::historical_state::VersionScopingSupport;

pub(crate) async fn handle_object_royalty(
    state: State<EngineStateApiState>,
    Json(request): Json<models::ObjectRoyaltyRequest>,
) -> Result<Json<models::ObjectRoyaltyResponse>, ResponseError> {
    let mapping_context = MappingContext::new(&state.network);
    let extraction_context = ExtractionContext::new(&state.network);

    let node_id = extract_address_as_node_id(&extraction_context, &request.entity_address)
        .map_err(|err| err.into_response_error("entity_address"))?;

    let requested_state_version =
        extract_opt_ledger_state_selector(request.at_ledger_state.as_deref())
            .map_err(|err| err.into_response_error("at_ledger_state"))?;

    let database = state
        .state_manager
        .database
        .snapshot()
        .scoped_at(requested_state_version)?;

    let loader = ObjectRoyaltyLoader::new(&database);
    let method_amounts = loader.load_method_amounts(&node_id)?;

    let header = database.proving_ledger_header();

    Ok(Json(models::ObjectRoyaltyResponse {
        at_ledger_state: Box::new(to_api_ledger_state_summary(&mapping_context, &header)?),
        method_royalties: method_amounts
            .into_iter()
            .map(|method_amount| to_api_method_royalty(&mapping_context, method_amount))
            .collect::<Result<Vec<_>, _>>()?,
    }))
}

fn to_api_method_royalty(
    _context: &MappingContext,
    method_amount: MethodRoyaltyAmount,
) -> Result<models::ObjectMethodRoyalty, MappingError> {
    let MethodRoyaltyAmount {
        name,
        for_component,
        for_package,
    } = method_amount;
    Ok(models::ObjectMethodRoyalty {
        name,
        component_royalty_amount: to_api_royalty_amount(&for_component).map(Box::new),
        package_royalty_amount: to_api_royalty_amount(&for_package).map(Box::new),
    })
}
