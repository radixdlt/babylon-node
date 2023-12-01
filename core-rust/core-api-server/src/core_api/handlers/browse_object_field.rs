use crate::core_api::*;

use radix_engine::types::*;

use std::ops::Deref;

pub(crate) async fn handle_browse_object_field(
    state: State<CoreApiState>,
    Json(request): Json<models::BrowseObjectFieldRequest>,
) -> Result<Json<models::BrowseObjectFieldResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;
    let mapping_context = MappingContext::new(&state.network);
    let extraction_context = ExtractionContext::new(&state.network);

    let node_id = extract_address_as_node_id(&extraction_context, &request.entity_address)
        .map_err(|err| err.into_response_error("entity_address"))?;
    let module_id = request
        .module_id
        .map(|module_id| extract_api_module_id(&module_id))
        .unwrap_or(ModuleId::Main);
    let field_coordinate = extract_field_coordinate(request.field_name, request.field_index)
        .map_err(|err| err.into_response_error("field_name or field_index"))?;

    let database = state.state_manager.database.read_current();

    let meta_loader = EngineStateMetaLoader::new(database.deref());
    let module_state_meta = meta_loader.load_object_module_state_meta(&node_id, module_id)?;
    let field_meta = match field_coordinate {
        FieldCoordinate::Name(field_name) => module_state_meta.field_by_name(field_name),
        FieldCoordinate::Index(field_index) => module_state_meta.field_by_index(field_index),
    }?;

    let data_loader = EngineStateDataLoader::new(database.deref());
    let field_data = data_loader.load_field_value(&node_id, module_id, field_meta)?;
    let programmatic_json = field_data.into_programmatic_json(&mapping_context)?;

    let header = read_current_ledger_header(database.deref());

    Ok(models::BrowseObjectFieldResponse {
        at_ledger_state: Box::new(to_api_ledger_state_summary(&mapping_context, &header)?),
        content: Box::new(models::BrowseObjectFieldResponseContent { programmatic_json }),
    })
    .map(Json)
}

fn extract_field_coordinate(
    field_name: Option<String>,
    field_index: Option<i32>,
) -> Result<FieldCoordinate, ExtractionError> {
    if let Some(field_name) = field_name {
        if field_index.is_some() {
            Err(ExtractionError::InvalidFieldAlternativesUsage)
        } else {
            Ok(FieldCoordinate::Name(field_name))
        }
    } else if let Some(field_index) = field_index {
        Ok(FieldCoordinate::Index(extract_api_u8_as_i32(field_index)?))
    } else {
        Err(ExtractionError::InvalidFieldAlternativesUsage)
    }
}

enum FieldCoordinate {
    Name(String),
    Index(u8),
}
