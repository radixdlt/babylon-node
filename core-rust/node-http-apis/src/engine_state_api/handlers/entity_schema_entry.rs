use crate::engine_state_api::*;

use radix_engine::types::*;

use std::ops::Deref;

pub(crate) async fn handle_entity_schema_entry(
    state: State<EngineStateApiState>,
    Json(request): Json<models::EntitySchemaEntryRequest>,
) -> Result<Json<models::EntitySchemaEntryResponse>, ResponseError> {
    let mapping_context = MappingContext::new(&state.network);
    let extraction_context = ExtractionContext::new(&state.network);

    let node_id = extract_address_as_node_id(&extraction_context, &request.entity_address)
        .map_err(|err| err.into_response_error("entity_address"))?;
    let schema_hash = extract_schema_hash(request.schema_hash.as_str())
        .map_err(|err| err.into_response_error("schema_hash"))?;

    let database = state.state_manager.database.read_current();

    let data_loader = EngineStateDataLoader::new(database.deref());
    let versioned_schema_data = data_loader.load_schema(&SchemaReference {
        node_id,
        schema_hash,
    })?;
    let programmatic_json = versioned_schema_data.into_programmatic_json(&mapping_context)?;

    let header = read_current_ledger_header(database.deref());

    Ok(Json(models::EntitySchemaEntryResponse {
        at_ledger_state: Box::new(to_api_ledger_state_summary(&mapping_context, &header)?),
        content: Box::new(models::EntitySchemaEntryResponseContent { programmatic_json }),
    }))
}
