use crate::core_api::*;
use radix_engine::system::node_modules::access_rules::MethodAccessRulesSubstate;
use radix_engine::types::PackageOffset;
use radix_engine_interface::types::{
    AccessRulesOffset, ACCESS_RULES_BASE_MODULE, OBJECT_BASE_MODULE,
};
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

    let package_info: PackageInfoSubstate = read_mandatory_substate(
        database.deref(),
        package_address.as_node_id(),
        OBJECT_BASE_MODULE,
        &PackageOffset::Info.into(),
    )?;

    let package_royalty: PackageRoyaltySubstate = read_mandatory_substate(
        database.deref(),
        package_address.as_node_id(),
        OBJECT_BASE_MODULE,
        &PackageOffset::Royalty.into(),
    )?;

    let method_access_rules_substate: MethodAccessRulesSubstate = read_mandatory_substate(
        database.deref(),
        package_address.as_node_id(),
        ACCESS_RULES_BASE_MODULE,
        &AccessRulesOffset::AccessRules.into(),
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
