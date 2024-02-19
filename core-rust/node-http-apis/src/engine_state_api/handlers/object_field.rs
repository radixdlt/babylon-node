use crate::engine_state_api::*;

use crate::engine_prelude::*;

use std::ops::Deref;

pub(crate) async fn handle_object_field(
    state: State<EngineStateApiState>,
    Json(request): Json<models::ObjectFieldRequest>,
) -> Result<Json<models::ObjectFieldResponse>, ResponseError> {
    let mapping_context =
        MappingContext::new(&state.network).with_sbor_formats(request.sbor_format_options);
    let extraction_context = ExtractionContext::new(&state.network);

    let node_id = extract_address_as_node_id(&extraction_context, &request.entity_address)
        .map_err(|err| err.into_response_error("entity_address"))?;
    let module_id = request
        .attached_module_id
        .map(|module_id| extract_api_attached_module_id(&module_id).into())
        .unwrap_or(ModuleId::Main);
    let field_input = extract_api_rich_index_input(request.field_name, request.field_index)
        .map_err(|err| err.into_response_error("field_name or field_index"))?;

    let database = state.state_manager.database.snapshot();

    let meta_loader = EngineStateMetaLoader::new(database.deref());
    let module_state_meta = meta_loader.load_object_module_state_meta(&node_id, module_id)?;
    let field_meta = match field_input {
        RichIndexInput::Name(name) => module_state_meta.field_by_name(name),
        RichIndexInput::Index(index) => module_state_meta.field_by_index(index),
    }?;

    let data_loader = EngineStateDataLoader::new(database.deref());
    let field_data = data_loader.load_field_value(&node_id, module_id, field_meta)?;

    let header = read_current_ledger_header(database.deref());

    Ok(Json(models::ObjectFieldResponse {
        at_ledger_state: Box::new(to_api_ledger_state_summary(&mapping_context, &header)?),
        content: Box::new(to_api_sbor_data(&mapping_context, field_data)?),
    }))
}
