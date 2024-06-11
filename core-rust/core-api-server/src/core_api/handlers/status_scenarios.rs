use crate::core_api::*;

use crate::engine_prelude::*;

use state_manager::store::traits::scenario::{
    ExecutedScenario, ExecutedScenarioStore, ExecutedScenarioTransaction, ScenarioSequenceNumber,
};

#[tracing::instrument(skip(state))]
pub(crate) async fn handle_status_scenarios(
    state: State<CoreApiState>,
    Json(request): Json<models::ScenariosRequest>,
) -> Result<Json<models::ScenariosResponse>, ResponseError<()>> {
    assert_matching_network(&request.network, &state.network)?;
    let context = MappingContext::new(&state.network);

    let database = state.state_manager.database.access_direct();
    let scenarios = database.list_all_scenarios();

    Ok(Json(models::ScenariosResponse {
        executed_scenarios: scenarios
            .iter()
            .map(|(number, scenario)| to_api_executed_scenario(&context, *number, scenario))
            .collect::<Result<Vec<_>, _>>()?,
    }))
}

pub fn to_api_executed_scenario(
    context: &MappingContext,
    number: ScenarioSequenceNumber,
    scenario: &ExecutedScenario,
) -> Result<models::ExecutedScenario, MappingError> {
    Ok(models::ExecutedScenario {
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
            .map(|address| {
                (
                    address.logical_name.to_owned(),
                    address.rendered_address.to_owned(),
                )
            })
            .collect(),
    })
}

pub fn to_api_scenario_transaction(
    context: &MappingContext,
    transaction: &ExecutedScenarioTransaction,
) -> Result<models::ExecutedScenarioTransaction, MappingError> {
    Ok(models::ExecutedScenarioTransaction {
        logical_name: transaction.logical_name.clone(),
        state_version: to_api_state_version(transaction.state_version)?,
        intent_hash: to_api_intent_hash(&transaction.intent_hash),
        intent_hash_bech32m: to_api_hash_bech32m(context, &transaction.intent_hash)?,
    })
}
