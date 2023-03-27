use crate::core_api::*;
use radix_engine::transaction::BalanceChange;
use state_manager::{
    jni::state_manager::ActualStateManager,
    store::traits::{QueryableProofStore, QueryableTransactionStore},
    CommittedTransactionIdentifiers, LedgerTransactionOutcome, LedgerTransactionReceipt,
};

#[tracing::instrument(skip(state), err(Debug))]
pub(crate) async fn handle_lts_stream_transactions_basic_outcomes(
    state: State<CoreApiState>,
    request: Json<models::LtsStreamTransactionsBasicOutcomesRequest>,
) -> Result<Json<models::LtsStreamTransactionsBasicOutcomesResponse>, ResponseError<()>> {
    core_api_read_handler(
        state,
        request,
        handle_lts_stream_transactions_basic_outcomes_internal,
    )
}

fn handle_lts_stream_transactions_basic_outcomes_internal(
    state_manager: &ActualStateManager,
    request: models::LtsStreamTransactionsBasicOutcomesRequest,
) -> Result<models::LtsStreamTransactionsBasicOutcomesResponse, ResponseError<()>> {
    assert_matching_network(&request.network, &state_manager.network)?;

    let from_state_version: u64 = extract_api_state_version(request.from_state_version)
        .map_err(|err| err.into_response_error("from_state_version"))?;

    let limit: u64 = request
        .limit
        .try_into()
        .map_err(|_| client_error("limit cannot be negative"))?;

    if limit == 0 {
        return Err(client_error("limit must be positive"));
    }

    if limit > MAX_STREAM_COUNT_PER_REQUEST.into() {
        return Err(client_error(format!(
            "limit must <= {MAX_STREAM_COUNT_PER_REQUEST}"
        )));
    }

    let max_state_version = state_manager.store().max_state_version();

    let txns = state_manager.store().get_committed_transaction_bundles(
        from_state_version,
        limit.try_into().expect("limit out of usize bounds"),
    );

    let mapping_context = MappingContext::new(&state_manager.network);

    let committed_transaction_outcomes = txns
        .into_iter()
        .map(|(_ledger_transaction, receipt, identifiers)| {
            Ok(to_api_lts_comitted_transaction_basic_outcome(
                &mapping_context,
                receipt,
                identifiers,
            )?)
        })
        .collect::<Result<Vec<models::LtsCommittedTransactionOutcome>, ResponseError<()>>>()?;

    let count: i32 = {
        let transaction_count = committed_transaction_outcomes.len();
        if transaction_count > MAX_STREAM_COUNT_PER_REQUEST.into() {
            return Err(server_error("Too many transactions were loaded somehow"));
        }
        transaction_count
            .try_into()
            .map_err(|_| server_error("Unexpected error mapping small usize to i32"))?
    };

    Ok(models::LtsStreamTransactionsBasicOutcomesResponse {
        from_state_version: to_api_state_version(from_state_version)?,
        count,
        max_ledger_state_version: to_api_state_version(max_state_version)?,
        committed_transaction_outcomes,
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_lts_comitted_transaction_basic_outcome(
    context: &MappingContext,
    receipt: LedgerTransactionReceipt,
    identifiers: CommittedTransactionIdentifiers,
) -> Result<models::LtsCommittedTransactionOutcome, MappingError> {
    let status = match receipt.outcome {
        LedgerTransactionOutcome::Success(_) => models::LtsCommittedTransactionStatus::Succeeded,
        LedgerTransactionOutcome::Failure(_) => models::LtsCommittedTransactionStatus::Failed,
    };

    let fungible_entity_balance_changes = receipt
        .state_update_summary
        .balance_changes
        .iter()
        .map(
            |(address, resource_changes)| models::LtsEntityFungibleBalanceChanges {
                address: to_api_address(context, address),
                fungible_resource_balance_changes: resource_changes
                    .iter()
                    .filter_map(|(resource_address, balance_change)| match balance_change {
                        BalanceChange::Fungible(balance_change) => {
                            Some(models::LtsFungibleResourceBalanceChange {
                                fungible_resource_address: to_api_resource_address(
                                    context,
                                    resource_address,
                                ),
                                balance_change: to_api_decimal(balance_change),
                            })
                        }
                        BalanceChange::NonFungible { .. } => None,
                    })
                    .collect(),
            },
        )
        .collect();

    Ok(models::LtsCommittedTransactionOutcome {
        state_version: to_api_state_version(identifiers.state_version)?,
        accumulator_hash: to_api_accumulator_hash(&identifiers.accumulator_hash),
        status,
        fungible_entity_balance_changes,
    })
}
