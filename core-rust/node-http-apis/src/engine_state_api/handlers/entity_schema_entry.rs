use crate::engine_state_api::*;

use crate::engine_prelude::*;

use state_manager::historical_state::VersionScopingSupport;

pub(crate) async fn handle_entity_schema_entry(
    state: State<EngineStateApiState>,
    Json(request): Json<models::EntitySchemaEntryRequest>,
) -> Result<Json<models::EntitySchemaEntryResponse>, ResponseError> {
    let mapping_context =
        MappingContext::new(&state.network).with_sbor_formats(request.sbor_format_options);
    let extraction_context = ExtractionContext::new(&state.network);

    let node_id = extract_address_as_node_id(&extraction_context, &request.entity_address)
        .map_err(|err| err.into_response_error("entity_address"))?;
    let schema_hash = extract_schema_hash(request.schema_hash.as_str())
        .map_err(|err| err.into_response_error("schema_hash"))?;
    let requested_state_version =
        extract_opt_ledger_state_selector(request.at_ledger_state.as_deref())
            .map_err(|err| err.into_response_error("at_ledger_state"))?;

    let database = state
        .state_manager
        .database
        .snapshot()
        .scoped_at(requested_state_version)?;

    let data_loader = EngineStateDataLoader::new(&database);
    let versioned_schema_data = data_loader.load_schema(&SchemaReference {
        node_id,
        schema_hash,
    })?;

    let header = database.proving_ledger_header();

    Ok(Json(models::EntitySchemaEntryResponse {
        at_ledger_state: Box::new(to_api_ledger_state_summary(&mapping_context, &header)?),
        content: Box::new(to_api_sbor_data(&mapping_context, versioned_schema_data)?),
    }))
}
