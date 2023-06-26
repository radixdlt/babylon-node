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

    let package_royalty_accumulator: PackageRoyaltyAccumulatorSubstate =
        read_mandatory_main_field_substate(
            database.deref(),
            package_address.as_node_id(),
            &PackageField::Royalty.into(),
        )?;

    let owner_role_substate: OwnerRole = read_mandatory_substate(
        database.deref(),
        package_address.as_node_id(),
        ACCESS_RULES_FIELDS_PARTITION,
        &AccessRulesField::OwnerRole.into(),
    )?;

    Ok(models::StatePackageResponse {
        royalty: Some(to_api_package_royalty_accumulator_substate(
            &mapping_context,
            &package_royalty_accumulator,
        )?),
        owner_role: Some(to_api_owner_role_substate(
            &mapping_context,
            &owner_role_substate,
        )?),
    })
    .map(Json)
}
