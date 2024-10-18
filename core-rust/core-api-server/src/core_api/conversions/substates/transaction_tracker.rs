use super::super::*;
use super::*;
use crate::core_api::models;
use crate::engine_prelude::*;

pub fn to_api_transaction_tracker_substate(
    context: &MappingContext,
    substate: &FieldSubstate<TransactionTrackerSubstate>,
) -> Result<models::Substate, MappingError> {
    Ok(field_substate!(
        substate,
        TransactionTrackerFieldState,
        value => {
            let TransactionTrackerSubstateV1 {
                start_epoch,
                start_partition,
                partition_range_start_inclusive,
                partition_range_end_inclusive,
                epochs_per_partition,
            } = value.v1()
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
    assert_key_type!(
        typed_key,
        TypedSubstateKey::MainModule(
            TypedMainModuleSubstateKey::TransactionTrackerCollectionEntry(intent_hash)
        )
    );
    Ok(key_value_store_mandatory_substate!(
        substate,
        TransactionTrackerCollectionEntry,
        models::TransactionIdKey {
            intent_hash: to_api_transaction_intent_hash(intent_hash),
            intent_hash_bech32m: to_api_hash_bech32m(context, intent_hash)?,
        },
        value => {
            status: match value {
                TransactionStatus::V1(TransactionStatusV1::CommittedSuccess) => {
                    models::TransactionTrackerTransactionStatus::CommittedSuccess
                }
                TransactionStatus::V1(TransactionStatusV1::CommittedFailure) => {
                    models::TransactionTrackerTransactionStatus::CommittedFailure
                }
                TransactionStatus::V1(TransactionStatusV1::Cancelled) => {
                    models::TransactionTrackerTransactionStatus::Cancelled
                }
            },
        }
    ))
}
