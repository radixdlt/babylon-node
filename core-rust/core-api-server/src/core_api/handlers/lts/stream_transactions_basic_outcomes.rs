use crate::core_api::*;
use radix_engine::transaction::BalanceChange;
use state_manager::{
    jni::state_manager::ActualStateManager,
    store::traits::{QueryableProofStore, QueryableTransactionStore},
    CommittedTransactionIdentifiers, LedgerTransactionOutcome, LedgerTransactionReceipt,
};

#[tracing::instrument(skip(state), err(Debug))]
pub(crate) async fn handle_rc_stream_transactions_basic_outcomes(
    state: State<CoreApiState>,
    request: Json<models::LtsStreamTransactionsBasicOutcomesRequest>,
) -> Result<Json<models::LtsStreamTransactionsBasicOutcomesResponse>, ResponseError<()>> {
    core_api_read_handler(
        state,
        request,
        handle_rc_stream_transactions_basic_outcomes_internal,
    )
}

fn handle_rc_stream_transactions_basic_outcomes_internal(
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

    let api_txns = txns
        .into_iter()
        .map(|(_ledger_transaction, receipt, identifiers)| {
            Ok(to_api_lts_comitted_transaction_basic_outcome(
                &mapping_context,
                receipt,
                identifiers,
            )?)
        })
        .collect::<Result<Vec<models::LtsCommittedTransactionBasicOutcome>, ResponseError<()>>>()?;

    let start_state_version = if api_txns.is_empty() {
        0
    } else {
        from_state_version
    };

    let count: i32 = {
        let transaction_count = api_txns.len();
        if transaction_count > MAX_STREAM_COUNT_PER_REQUEST.into() {
            return Err(server_error("Too many transactions were loaded somehow"));
        }
        transaction_count
            .try_into()
            .map_err(|_| server_error("Unexpected error mapping small usize to i32"))?
    };

    Ok(models::LtsStreamTransactionsBasicOutcomesResponse {
        from_state_version: to_api_state_version(start_state_version)?,
        count,
        max_ledger_state_version: to_api_state_version(max_state_version)?,
        committed_transaction_basic_outcomes: api_txns,
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_lts_comitted_transaction_basic_outcome(
    context: &MappingContext,
    receipt: LedgerTransactionReceipt,
    identifiers: CommittedTransactionIdentifiers,
) -> Result<models::LtsCommittedTransactionBasicOutcome, MappingError> {
    Ok(models::LtsCommittedTransactionBasicOutcome {
        state_version: to_api_state_version(identifiers.state_version)?,
        accumulator_hash: to_api_accumulator_hash(&identifiers.accumulator_hash),
        basic_outcome: Box::new(to_api_lts_basic_outcome(context, receipt)?),
    })
}

#[tracing::instrument(skip_all)]
pub fn to_api_lts_basic_outcome(
    context: &MappingContext,
    receipt: LedgerTransactionReceipt,
) -> Result<models::LtsBasicOutcome, MappingError> {
    let status = match receipt.outcome {
        LedgerTransactionOutcome::Success(_) => models::LtsCommittedTransactionStatus::Succeeded,
        LedgerTransactionOutcome::Failure(_) => models::LtsCommittedTransactionStatus::Failed,
    };

    let account_balance_updates = receipt
        .state_update_summary
        .balance_changes
        .iter()
        .map(
            |(address, resource_changes)| models::LtsAccountBalanceUpdates {
                account_address: to_api_address(context, address),
                resource_balance_updates: resource_changes
                    .iter()
                    .filter_map(|(resource_address, balance_change)| match balance_change {
                        BalanceChange::Fungible(delta) => Some(models::LtsResourceBalanceUpdate {
                            resource_address: to_api_resource_address(context, resource_address),
                            balance_delta: to_api_decimal(delta),
                        }),
                        BalanceChange::NonFungible {
                            added: _added,
                            removed: _removed,
                        } => None,
                    })
                    .collect(),
            },
        )
        .collect();

    Ok(models::LtsBasicOutcome {
        status,
        account_balance_updates,
    })
}
