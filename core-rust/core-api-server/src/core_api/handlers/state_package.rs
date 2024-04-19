use crate::core_api::*;
use crate::engine_prelude::*;

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

    let database = state.state_manager.database.snapshot();

    let owner_role_substate = read_optional_substate(
        database.deref(),
        package_address.as_node_id(),
        RoleAssignmentPartitionOffset::Field.as_partition(ROLE_ASSIGNMENT_BASE_PARTITION),
        &RoleAssignmentField::Owner.into(),
    )
    .ok_or_else(|| not_found_error("Package not found".to_string()))?;

    let package_royalty_accumulator = read_optional_main_field_substate(
        database.deref(),
        package_address.as_node_id(),
        &PackageField::RoyaltyAccumulator.into(),
    );

    let header = read_current_ledger_header(database.deref());

    Ok(Json(models::StatePackageResponse {
        at_ledger_state: Box::new(to_api_ledger_state_summary(
            &mapping_context,
            &header.into(),
        )?),
        owner_role: Some(to_api_owner_role_substate(
            &mapping_context,
            &owner_role_substate,
        )?),
        royalty: package_royalty_accumulator
            .map(|substate| -> Result<_, MappingError> {
                Ok(Box::new(to_api_package_royalty_accumulator_substate(
                    &mapping_context,
                    &substate,
                )?))
            })
            .transpose()?,
    }))
}
