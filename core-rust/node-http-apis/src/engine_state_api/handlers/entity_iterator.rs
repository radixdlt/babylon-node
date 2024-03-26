use crate::engine_state_api::*;

use crate::engine_prelude::*;

use state_manager::historical_state::resolve_effective_state_version;
use state_manager::store::traits::indices::CreationId;
use state_manager::store::traits::ConfigurableDatabase;
use state_manager::traits::indices::ReNodeListingIndex;
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

    let database = state.state_manager.database.snapshot();
    if !database.are_re_node_listing_indices_enabled() {
        return Err(ResponseError::new(
            StatusCode::CONFLICT,
            "Required Node feature is not enabled",
        )
        .with_internal_message(
            "Missing `db.re_node_listing_indices.enable = true` Node configuration flag",
        ));
    }
    let effective_state_version =
        resolve_effective_state_version(database.deref(), effective_filter.at_state_version)?;

    let page = paging_support.get_page(create_listing_call(database.deref(), effective_filter))?;

    let header = read_proving_ledger_header(database.deref(), effective_state_version);

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

// Note: the usual "listing call" (expected by the paging infra) looks like:
// `impl FnOnce(Option<K>) -> Result<impl Iterator<Item = T>, E>`
// For this endpoint, however, the chained nature of filters force us to apply them selectively (and
// thus operate on fully polymorphic functions and iterators).
// The type aliases below are only needed to reduce the super-long typing and simplify lifetimes.
type BoxListingCall<'l, A, T, E> = Box<dyn FnOnce(Option<&A>) -> Result<BoxIter<'l, T>, E> + 'l>;
type BoxIter<'l, T> = Box<dyn Iterator<Item = T> + 'l>;

fn create_listing_call(
    database: &impl ReNodeListingIndex,
    effective_filter: EffectiveFilter,
) -> BoxListingCall<CreationId, EntitySummary, EngineStateBrowsingError> {
    let EffectiveFilter {
        explicit_filter,
        at_state_version,
    } = effective_filter;
    let current_version_listing_call =
        create_current_version_listing_call(database, explicit_filter);
    apply_historical_version_filter(current_version_listing_call, at_state_version)
}

fn create_current_version_listing_call(
    database: &impl ReNodeListingIndex,
    explicit_filter: Option<ExplicitFilter>,
) -> BoxListingCall<CreationId, EntitySummary, EngineStateBrowsingError> {
    let entity_lister = EngineEntityLister::new(database);
    match explicit_filter {
        None => Box::new(move |from| {
            entity_lister
                .iter_created_entities(all_entity_types(), from)
                .map(|iterator| Box::new(iterator) as BoxIter<EntitySummary>)
        }),
        Some(ExplicitFilter::OneOfEntityTypes(entity_types)) => Box::new(move |from| {
            entity_lister
                .iter_created_entities(entity_types.into_iter(), from)
                .map(|iterator| Box::new(iterator) as BoxIter<EntitySummary>)
        }),
        Some(ExplicitFilter::Blueprint(blueprint_id)) => Box::new(move |from| {
            entity_lister
                .iter_blueprint_entities(&blueprint_id, from)
                .map(|iterator| Box::new(iterator) as BoxIter<EntitySummary>)
        }),
    }
}

fn apply_historical_version_filter(
    listing_call: BoxListingCall<CreationId, EntitySummary, EngineStateBrowsingError>,
    at_state_version: Option<StateVersion>,
) -> BoxListingCall<CreationId, EntitySummary, EngineStateBrowsingError> {
    match at_state_version {
        None => listing_call,
        Some(at_state_version) => Box::new(move |from| {
            listing_call(from).map(|iterator| {
                Box::new(iterator.take_while(move |summary| {
                    summary.creation_id.state_version <= at_state_version
                })) as BoxIter<EntitySummary>
            })
        }),
    }
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
