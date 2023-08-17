use super::super::*;
use super::*;
use crate::core_api::models;

use radix_engine::types::*;
use radix_engine_queries::typed_substate_layout::*;

pub fn to_api_transaction_tracker_substate(
    context: &MappingContext,
    substate: &FieldSubstate<TransactionTrackerSubstate>,
) -> Result<models::Substate, MappingError> {
    Ok(field_substate!(
        substate,
        TransactionTrackerFieldState,
        TransactionTrackerSubstate {
            start_epoch,
            start_partition,
            partition_range_start_inclusive,
            partition_range_end_inclusive,
            epochs_per_partition,
        },
        Value {
            start_epoch: to_api_epoch(context, Epoch::of(*start_epoch))?,
            start_partition: to_api_u8_as_i32(*start_partition),
            partition_range_start_inclusive: to_api_u8_as_i32(*partition_range_start_inclusive),
            partition_range_end_inclusive: to_api_u8_as_i32(*partition_range_end_inclusive),
            epochs_per_partition: to_api_epoch(context, Epoch::of(*epochs_per_partition))?,
        }
    ))
}

pub fn to_api_transaction_tracker_collection_entry(
    context: &MappingContext,
    typed_key: &TypedSubstateKey,
    substate: &KeyValueEntrySubstate<TransactionStatus>,
) -> Result<models::Substate, MappingError> {
    let TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::TransactionTrackerCollectionEntry(intent_hash)) = typed_key else {
        return Err(MappingError::MismatchedSubstateKeyType { message: "Transaction Tracker Collection Key".to_string() });
    };
    Ok(key_value_store_mandatory_substate!(
        substate,
        TransactionTrackerCollectionEntry,
        models::TransactionIdKey {
            intent_hash: to_api_intent_hash(intent_hash),
            intent_hash_bech32m: to_api_hash_bech32m(context, intent_hash)?,
        },
        value => {
            status: match value {
                TransactionStatus::CommittedSuccess => {
                    models::TransactionTrackerTransactionStatus::CommittedSuccess
                }
                TransactionStatus::CommittedFailure => {
                    models::TransactionTrackerTransactionStatus::CommittedFailure
                }
                TransactionStatus::Cancelled => {
                    models::TransactionTrackerTransactionStatus::Cancelled
                }
            },
        }
    ))
}
