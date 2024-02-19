use crate::engine_state_api::*;

use crate::engine_prelude::*;

use std::ops::Deref;

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

    let database = state.state_manager.database.snapshot();

    let data_loader = EngineStateDataLoader::new(database.deref());
    let versioned_schema_data = data_loader.load_schema(&SchemaReference {
        node_id,
        schema_hash,
    })?;

    let header = read_current_ledger_header(database.deref());

    Ok(Json(models::EntitySchemaEntryResponse {
        at_ledger_state: Box::new(to_api_ledger_state_summary(&mapping_context, &header)?),
        content: Box::new(to_api_sbor_data(&mapping_context, versioned_schema_data)?),
    }))
}
