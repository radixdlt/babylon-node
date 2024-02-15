use crate::engine_state_api::*;
use std::iter::once;

use crate::engine_prelude::*;

use state_manager::store::traits::indices::CreationId;
use state_manager::store::traits::ConfigurableDatabase;
use std::ops::Deref;

use crate::engine_state_api::handlers::HandlerPagingSupport;

pub(crate) async fn handle_entity_iterator(
    state: State<EngineStateApiState>,
    Json(request): Json<models::EntityIteratorRequest>,
) -> Result<Json<models::EntityIteratorResponse>, ResponseError> {
    let mapping_context = MappingContext::new(&state.network);
    let extraction_context = ExtractionContext::new(&state.network);
    let paging_support = HandlerPagingSupport::new(
        request.max_page_size,
        request.continuation_token,
        &request.filter,
    );

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
    let page = match request.filter.map(|boxed| *boxed) {
        None => paging_support
            .get_page(|from| entity_lister.iter_created_entities(all_entity_types(), from))?,
        Some(models::EntityIteratorFilter::SystemTypeFilter { system_type }) => paging_support
            .get_page(|from| {
                entity_lister.iter_created_entities(system_type_to_entity_types(&system_type), from)
            })?,
        Some(models::EntityIteratorFilter::EntityTypeFilter { entity_type }) => paging_support
            .get_page(|from| {
                entity_lister.iter_created_entities(once(extract_entity_type(entity_type)), from)
            })?,
        Some(models::EntityIteratorFilter::BlueprintFilter { blueprint }) => {
            let blueprint_id = extract_blueprint_id(&extraction_context, &blueprint)
                .map_err(|err| err.into_response_error("blueprint"))?;
            paging_support
                .get_page(|from| entity_lister.iter_blueprint_entities(&blueprint_id, from))?
        }
    };

    let header = read_current_ledger_header(database.deref());

    Ok(Json(models::EntityIteratorResponse {
        at_ledger_state: Box::new(to_api_ledger_state_summary(&mapping_context, &header)?),
        page: page
            .items
            .into_iter()
            .map(|entity_summary| to_api_listed_entity_item(&mapping_context, &entity_summary))
            .collect::<Result<Vec<_>, _>>()?,
        continuation_token: page.continuation_token,
    }))
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
        system_type: entity_type_to_system_type(&entity_type),
        entity_type: to_api_entity_type(entity_type),
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

fn extract_blueprint_id(
    extraction_context: &ExtractionContext,
    reference: &models::UnversionedBlueprintReference,
) -> Result<BlueprintId, ExtractionError> {
    let models::UnversionedBlueprintReference {
        package_address,
        blueprint_name,
    } = reference;
    Ok(BlueprintId {
        package_address: extract_package_address(extraction_context, package_address)?,
        blueprint_name: blueprint_name.to_string(),
    })
}

fn all_entity_types() -> impl Iterator<Item = EntityType> {
    (0..=u8::MAX).filter_map(EntityType::from_repr)
}

fn entity_type_to_system_type(entity_type: &EntityType) -> models::SystemType {
    if entity_type.is_internal_kv_store() {
        models::SystemType::KeyValueStore
    } else {
        models::SystemType::Object
    }
}

fn system_type_to_entity_types(
    system_type: &models::SystemType,
) -> impl Iterator<Item = EntityType> {
    let system_type = *system_type;
    all_entity_types()
        .filter(move |entity_type| entity_type_to_system_type(entity_type) == system_type)
}
