use crate::core_api::*;
use radix_engine::types::*;
use state_manager::store::traits::QueryableProofStore;
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

    let database = state.state_manager.database.read();

    let owner_role_substate = read_optional_substate(
        database.deref(),
        package_address.as_node_id(),
        ROLE_ASSIGNMENT_FIELDS_PARTITION,
        &RoleAssignmentField::OwnerRole.into(),
    )
    .ok_or_else(|| not_found_error("Package not found".to_string()))?;

    let package_royalty_accumulator = read_optional_main_field_substate(
        database.deref(),
        package_address.as_node_id(),
        &PackageField::Royalty.into(),
    );

    let header = database
        .get_last_proof()
        .expect("proof for outputted state must exist")
        .ledger_header;

    Ok(models::StatePackageResponse {
        at_ledger_state: Box::new(to_api_ledger_state_summary(&mapping_context, &header)?),
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
    })
    .map(Json)
}
