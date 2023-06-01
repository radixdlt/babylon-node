use crate::core_api::*;
use radix_engine::types::*;
use radix_engine_queries::typed_substate_layout::*;
use std::ops::Deref;

pub(crate) async fn handle_state_package(
    state: State<CoreApiState>,
    Json(request): Json<models::StatePackageRequest>,
) -> Result<Json<models::StatePackageResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;
    let mapping_context = MappingContext::new(&state.network);
    let extraction_context = ExtractionContext::new(&state.network);

    let package_address = extract_package_address(&extraction_context, &request.package_address)
        .map_err(|err| err.into_response_error("package_address"))?;

    let database = state.database.read();

    let package_info: PackageInfoSubstate = read_optional_main_field_substate(
        database.deref(),
        package_address.as_node_id(),
        &PackageField::Info.into(),
    )
    .ok_or_else(|| not_found_error("Package not found".to_string()))?;

    let package_royalty: PackageRoyaltySubstate = read_mandatory_main_field_substate(
        database.deref(),
        package_address.as_node_id(),
        &PackageField::Royalty.into(),
    )?;

    let method_access_rules_substate = read_mandatory_substate(
        database.deref(),
        package_address.as_node_id(),
        ACCESS_RULES_FIELD_PARTITION,
        &AccessRulesField::AccessRules.into(),
    )?;

    Ok(models::StatePackageResponse {
        info: Some(to_api_package_info_substate(
            &mapping_context,
            &package_info,
        )?),
        royalty: Some(to_api_package_royalty_substate(
            &mapping_context,
            &package_royalty,
        )?),
        access_rules: Some(to_api_method_access_rules_substate(
            &mapping_context,
            &method_access_rules_substate,
        )?),
    })
    .map(Json)
}
