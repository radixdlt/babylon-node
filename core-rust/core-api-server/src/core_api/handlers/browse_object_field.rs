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
    let field_input = extract_api_rich_index_input(request.field_name, request.field_index)
        .map_err(|err| err.into_response_error("field_name or field_index"))?;

    let database = state.state_manager.database.read_current();

    let meta_loader = EngineStateMetaLoader::new(database.deref());
    let module_state_meta = meta_loader.load_object_module_state_meta(&node_id, module_id)?;
    let field_meta = match field_input {
        RichIndexInput::Name(name) => module_state_meta.field_by_name(name),
        RichIndexInput::Index(index) => module_state_meta.field_by_index(index),
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
