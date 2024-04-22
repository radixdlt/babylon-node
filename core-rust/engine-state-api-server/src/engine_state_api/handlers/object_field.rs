use crate::engine_state_api::*;

use crate::engine_prelude::*;

use state_manager::historical_state::VersionScopingSupport;

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
        .map(|module_id| extract_attached_module_id(&module_id).into())
        .unwrap_or(ModuleId::Main);
    let field_input = extract_rich_index_input(request.field_name, request.field_index)
        .map_err(|err| err.into_response_error("field_name or field_index"))?;

    let requested_state_version =
        extract_opt_ledger_state_selector(request.at_ledger_state.as_deref())
            .map_err(|err| err.into_response_error("at_ledger_state"))?;

    let database = state
        .state_manager
        .database
        .snapshot()
        .scoped_at(requested_state_version)?;

    let meta_loader = EngineStateMetaLoader::new(&database);
    let module_state_meta = meta_loader.load_object_module_state_meta(&node_id, module_id)?;
    let field_meta = match field_input {
        RichIndexInput::Name(name) => module_state_meta.field_by_name(name),
        RichIndexInput::Index(index) => module_state_meta.field_by_index(index),
    }?;

    let data_loader = EngineStateDataLoader::new(&database);
    let field_data = data_loader.load_field_value(&node_id, module_id, field_meta)?;

    let ledger_state = database.at_ledger_state();

    Ok(Json(models::ObjectFieldResponse {
        at_ledger_state: Box::new(to_api_ledger_state_summary(
            &mapping_context,
            &ledger_state,
        )?),
        content: Box::new(to_api_sbor_data(&mapping_context, field_data)?),
    }))
}
