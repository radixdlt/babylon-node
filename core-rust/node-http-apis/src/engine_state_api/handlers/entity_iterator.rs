use crate::engine_state_api::*;

use crate::engine_prelude::*;

use state_manager::historical_state::VersionScopingSupport;
use state_manager::store::traits::indices::CreationId;
use state_manager::store::traits::ConfigurableDatabase;

use state_manager::StateVersion;
use std::ops::Deref;

use crate::engine_state_api::handlers::HandlerPagingSupport;

pub(crate) async fn handle_entity_iterator(
    state: State<EngineStateApiState>,
    Json(request): Json<models::EntityIteratorRequest>,
) -> Result<Json<models::EntityIteratorResponse>, ResponseError> {
    let mapping_context = MappingContext::new(&state.network);
    let extraction_context = ExtractionContext::new(&state.network);

    // Note: this endpoint formally accepts a single `filter` request field.
    // However, it also supports "historical state" - and luckily, when listing immutable entries
    // (i.e. "entity X created at version V"), the `at_ledger_state` field is effectively just
    // another filter (i.e. `entity.created_at_version <= request.at_state_version`). Hence:
    // - have to ensure it is stable across pages (see the `paging_support` instantiation below),
    // - we can simply pass it as part of the `create_listing_call()` parameter.
    let effective_filter =
        extract_effective_filter(&extraction_context, request.filter, request.at_ledger_state)?;

    let paging_support = HandlerPagingSupport::new_with_sbor_filter(
        request.max_page_size,
        request.continuation_token,
        &effective_filter,
    );

    let database = state
        .state_manager
        .database
        .snapshot()
        .scoped_at(effective_filter.at_state_version)?;
    if !database.are_re_node_listing_indices_enabled() {
        return Err(NodeFeatureDisabledError::new(
            "Entity listing",
            "db.re_node_listing_indices.enable",
        )
        .into());
    }

    let entity_lister = EngineEntityLister::new(&database);
    let page = match effective_filter.explicit_filter {
        None => paging_support
            .get_page(|from| entity_lister.iter_created_entities(all_entity_types(), from))?,
        Some(ExplicitFilter::OneOfEntityTypes(entity_types)) => paging_support
            .get_page(|from| entity_lister.iter_created_entities(entity_types.into_iter(), from))?,
        Some(ExplicitFilter::Blueprint(blueprint_id)) => paging_support
            .get_page(|from| entity_lister.iter_blueprint_entities(&blueprint_id, from))?,
    };

    let header = database.proving_ledger_header();

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

fn extract_effective_filter(
    extraction_context: &ExtractionContext,
    explicit_filter: Option<Box<models::EntityIteratorFilter>>,
    at_ledger_state: Option<Box<models::LedgerStateSelector>>,
) -> Result<EffectiveFilter, ResponseError> {
    Ok(EffectiveFilter {
        explicit_filter: explicit_filter
            .map(|explicit_filter| {
                Ok::<_, ResponseError>(match *explicit_filter {
                    models::EntityIteratorFilter::BlueprintFilter { blueprint } => {
                        ExplicitFilter::Blueprint(
                            extract_blueprint_id(extraction_context, blueprint.deref())
                                .map_err(|err| err.into_response_error("blueprint"))?,
                        )
                    }
                    models::EntityIteratorFilter::EntityTypeFilter { entity_type } => {
                        ExplicitFilter::OneOfEntityTypes(vec![extract_entity_type(entity_type)])
                    }
                    models::EntityIteratorFilter::SystemTypeFilter { system_type } => {
                        ExplicitFilter::OneOfEntityTypes(
                            system_type_to_entity_types(system_type).collect(),
                        )
                    }
                })
            })
            .transpose()?,
        at_state_version: extract_opt_ledger_state_selector(at_ledger_state.as_deref())
            .map_err(|err| err.into_response_error("at_ledger_state"))?,
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
    system_type: models::SystemType,
) -> impl Iterator<Item = EntityType> {
    all_entity_types()
        .filter(move |entity_type| entity_type_to_system_type(entity_type) == system_type)
}

// Note: see the comments at `handle_entity_iterator()` for motivation of the structs below:
#[derive(ScryptoEncode)]
struct EffectiveFilter {
    explicit_filter: Option<ExplicitFilter>,
    at_state_version: Option<StateVersion>,
}

#[derive(ScryptoEncode)]
enum ExplicitFilter {
    Blueprint(BlueprintId),
    OneOfEntityTypes(Vec<EntityType>),
}
