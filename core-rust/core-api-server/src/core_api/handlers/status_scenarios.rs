use crate::core_api::*;

use radix_engine_interface::prelude::*;

use state_manager::store::traits::scenario::{
    DescribedAddress, ExecutedGenesisScenario, ExecutedGenesisScenarioStore,
    ExecutedScenarioTransaction, ScenarioSequenceNumber,
};

#[tracing::instrument(skip(state))]
pub(crate) async fn handle_status_scenarios(
    state: State<CoreApiState>,
    Json(request): Json<models::ScenariosRequest>,
) -> Result<Json<models::ScenariosResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;
    let context = MappingContext::new(&state.network);

    let database = state.database.read();
    let scenarios = database.list_all_scenarios();

    Ok(models::ScenariosResponse {
        executed_scenarios: scenarios
            .iter()
            .map(|(number, scenario)| to_api_executed_scenario(&context, *number, scenario))
            .collect::<Result<Vec<_>, _>>()?,
    })
    .map(Json)
}

pub fn to_api_executed_scenario(
    context: &MappingContext,
    number: ScenarioSequenceNumber,
    scenario: &ExecutedGenesisScenario,
) -> Result<models::ExecutedGenesisScenario, MappingError> {
    Ok(models::ExecutedGenesisScenario {
        sequence_number: to_api_scenario_number(number)?,
        logical_name: scenario.logical_name.clone(),
        committed_transactions: scenario
            .committed_transactions
            .iter()
            .map(|transaction| to_api_scenario_transaction(context, transaction))
            .collect::<Result<Vec<_>, _>>()?,
        addresses: scenario
            .addresses
            .iter()
            .map(|address| to_api_described_address(context, address))
            .collect::<Result<Vec<_>, _>>()?,
    })
}

pub fn to_api_scenario_transaction(
    _context: &MappingContext,
    transaction: &ExecutedScenarioTransaction,
) -> Result<models::ExecutedScenarioTransaction, MappingError> {
    Ok(models::ExecutedScenarioTransaction {
        logical_name: transaction.logical_name.clone(),
        state_version: to_api_state_version(transaction.state_version)?,
        intent_hash: to_api_intent_hash(&transaction.intent_hash),
    })
}

pub fn to_api_described_address(
    _context: &MappingContext,
    address: &DescribedAddress,
) -> Result<models::DescribedAddress, MappingError> {
    Ok(models::DescribedAddress {
        logical_name: address.logical_name.clone(),
        address: address.rendered_address.clone(),
    })
}
