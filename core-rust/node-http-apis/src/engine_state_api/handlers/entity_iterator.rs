use crate::engine_state_api::*;

use radix_engine::types::*;

use crate::engine_state_api::handlers::default_paging_policy;
use state_manager::store::traits::indices::CreationId;
use state_manager::store::traits::ConfigurableDatabase;
use std::ops::Deref;

pub(crate) async fn handle_entity_iterator(
    state: State<EngineStateApiState>,
    Json(request): Json<models::EntityIteratorRequest>,
) -> Result<Json<models::EntityIteratorResponse>, ResponseError> {
    let mapping_context = MappingContext::new(&state.network);

    let requested_max_page_size = request
        .max_page_size
        .map(extract_api_max_page_size)
        .transpose()
        .map_err(|error| error.into_response_error("max_page_size"))?;
    let continuation_token = request
        .continuation_token
        .as_ref()
        .map(extract_api_sbor_hex_string)
        .transpose()
        .map_err(|error| error.into_response_error("continuation_token"))?;

    let database = state.state_manager.database.read_current();
    if !database.are_re_node_listing_indices_enabled() {
        return Err(ResponseError::new(
            StatusCode::CONFLICT,
            "Required Node feature is not enabled",
        )
        .with_internal_message(
            "Missing `db.re_node_listing_indices.enable = true` Node configuration flag",
        ));
    }

    let entity_lister = EngineEntityLister::new(database.deref());
    let page = OrderAgnosticPager::get_page(
        wrap_error_free(|from| entity_lister.iter_entities(from)),
        default_paging_policy(requested_max_page_size),
        continuation_token,
    )
    .expect("FnIterable is error-free");

    let header = read_current_ledger_header(database.deref());

    Ok(models::EntityIteratorResponse {
        at_ledger_state: Box::new(to_api_ledger_state_summary(&mapping_context, &header)?),
        page: page
            .items
            .into_iter()
            .map(|entity_summary| to_api_listed_entity_item(&mapping_context, &entity_summary))
            .collect::<Result<Vec<_>, _>>()?,
        continuation_token: page
            .continuation_token
            .map(|continuation_token| to_api_sbor_hex_string(&continuation_token))
            .transpose()?,
    })
    .map(Json)
}

impl HasKey<CreationId> for EntitySummary {
    fn as_key(&self) -> CreationId {
        self.creation_id.clone()
    }
}

fn to_api_listed_entity_item(
    context: &MappingContext,
    entity_summary: &EntitySummary,
) -> Result<models::ListedEntityItem, MappingError> {
    let EntitySummary {
        node_id,
        creation_id,
        blueprint_id,
    } = entity_summary;
    let entity_type = node_id.entity_type().ok_or(MappingError::EntityTypeError)?;
    Ok(models::ListedEntityItem {
        entity_type: to_api_entity_type(entity_type),
        system_type: if entity_type.is_internal_kv_store() {
            models::SystemType::KeyValueStore
        } else {
            models::SystemType::Object
        },
        is_global: node_id.is_global(),
        created_at_state_version: to_api_state_version(creation_id.state_version)?,
        entity_address: to_api_entity_address(context, node_id)?,
        blueprint: blueprint_id
            .clone()
            .map(|blueprint_id| to_api_unversioned_blueprint_reference(context, &blueprint_id))
            .transpose()?
            .map(Box::new),
    })
}

fn to_api_unversioned_blueprint_reference(
    context: &MappingContext,
    blueprint_id: &BlueprintId,
) -> Result<models::UnversionedBlueprintReference, MappingError> {
    let BlueprintId {
        package_address,
        blueprint_name,
    } = blueprint_id;
    Ok(models::UnversionedBlueprintReference {
        package_address: to_api_package_address(context, package_address)?,
        blueprint_name: blueprint_name.clone(),
    })
}
