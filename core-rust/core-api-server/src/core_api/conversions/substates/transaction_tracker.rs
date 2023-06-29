use super::*;
use super::super::*;
use crate::core_api::models;

use radix_engine::types::*;
use radix_engine_queries::typed_substate_layout::*;

pub fn to_api_transaction_tracker_substate(
    context: &MappingContext,
    substate: &TransactionTrackerSubstate,
) -> Result<models::Substate, MappingError> {
    let TransactionTrackerSubstate {
        start_epoch,
        start_partition,
        partition_range_start_inclusive,
        partition_range_end_inclusive,
        epochs_per_partition,
    } = substate;
    Ok(field_substate!(
        substate,
        TransactionTrackerFieldState,
        {
            start_epoch: to_api_epoch(context, Epoch::of(*start_epoch))?,
            start_partition: to_api_u8_as_i32(*start_partition),
            partition_range_start_inclusive: to_api_u8_as_i32(*partition_range_start_inclusive),
            partition_range_end_inclusive: to_api_u8_as_i32(*partition_range_end_inclusive),
            epochs_per_partition: to_api_epoch(context, Epoch::of(*epochs_per_partition))?,
        }
    ))
}

pub fn to_api_transaction_tracker_collection_entry(
    _context: &MappingContext,
    typed_key: &TypedSubstateKey,
    substate: &KeyValueEntrySubstate<TransactionStatus>,
) -> Result<models::Substate, MappingError> {
    let TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::TransactionTrackerCollectionEntry(intent_hash)) = typed_key else {
        return Err(MappingError::MismatchedSubstateKeyType { message: "Transaction Tracker Collection Key".to_string() });
    };
    Ok(key_value_store_substate!(
        substate,
        TransactionTrackerCollectionEntry,
        models::TransactionIdKey {
            intent_hash: to_api_hash(intent_hash.as_hash()),
        },
        {
            status: substate.value.as_ref().map(|status| match status {
                TransactionStatus::CommittedSuccess => {
                    models::TransactionTrackerTransactionStatus::CommittedSuccess
                }
                TransactionStatus::CommittedFailure => {
                    models::TransactionTrackerTransactionStatus::CommittedFailure
                }
                TransactionStatus::Cancelled => {
                    models::TransactionTrackerTransactionStatus::Cancelled
                }
            }),
        }
    ))
}