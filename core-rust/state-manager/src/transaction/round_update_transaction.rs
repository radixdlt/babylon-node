use radix_engine::types::*;

use crate::{LedgerHeader, RoundHistory};
use radix_engine_interface::api::node_modules::auth::AuthAddresses;
use radix_engine_interface::blueprints::consensus_manager::*;
use sbor::FixedEnumVariant;
use transaction::{define_raw_transaction_payload, model::*};

#[derive(Debug, Clone, Categorize, Encode, Decode, PartialEq, Eq)]
pub struct RoundUpdateTransactionV1 {
    pub proposer_timestamp_ms: i64,
    pub epoch: Epoch,
    pub round: Round,
    pub leader_proposal_history: LeaderProposalHistory,
}

impl RoundUpdateTransactionV1 {
    pub fn new(epoch_header: Option<&LedgerHeader>, round_history: &RoundHistory) -> Self {
        let validator_index_by_address = epoch_header
            .expect("at least genesis epoch is expected before any prepare")
            .next_epoch
            .as_ref()
            .expect("epoch header must contain next epoch information")
            .validator_set
            .iter()
            .enumerate()
            .map(|(validator_index, validator_info)| {
                (
                    validator_info.address,
                    ValidatorIndex::try_from(validator_index)
                        .expect("validator set size limit guarantees this"),
                )
            })
            .collect::<NonIterMap<_, _>>();
        RoundUpdateTransactionV1 {
            proposer_timestamp_ms: round_history.proposer_timestamp_ms,
            epoch: round_history.epoch,
            round: round_history.round,
            leader_proposal_history: LeaderProposalHistory {
                gap_round_leaders: round_history
                    .gap_round_leader_addresses
                    .iter()
                    .map(|leader_address| {
                        *validator_index_by_address
                            .get(leader_address)
                            .expect("gap round leader must belong to the validator set")
                    })
                    .collect::<Vec<_>>(),
                current_leader: *validator_index_by_address
                    .get(&round_history.proposer_address)
                    .expect("proposer must belong to the validator set"),
                is_fallback: round_history.is_fallback,
            },
        }
    }

    /// Note - we purposefully restrict what the content of a Round Update transaction can do
    /// so we convert it to instructions at run-time.
    pub fn create_instructions(&self) -> Vec<InstructionV1> {
        vec![InstructionV1::CallMethod {
            address: CONSENSUS_MANAGER.into(),
            method_name: CONSENSUS_MANAGER_NEXT_ROUND_IDENT.to_string(),
            args: to_manifest_value(&ConsensusManagerNextRoundInput {
                round: self.round,
                proposer_timestamp_ms: self.proposer_timestamp_ms,
                leader_proposal_history: self.leader_proposal_history.clone(),
            }),
        }]
    }

    pub fn prepare(&self) -> Result<PreparedRoundUpdateTransactionV1, PrepareError> {
        let prepared_instructions = InstructionsV1(self.create_instructions()).prepare_partial()?;
        let encoded_source = manifest_encode(&self)?;
        // Minor TODO - for a slight performance improvement, change this to be read from the decoder
        // As per the other hashes, don't include the prefix byte
        let source_hash = hash(&encoded_source[1..]);
        let instructions_hash = prepared_instructions.summary.hash;
        let round_update_hash = HashAccumulator::new()
            .update([
                TRANSACTION_HASHABLE_PAYLOAD_PREFIX,
                TransactionDiscriminator::V1RoundUpdate as u8,
            ])
            // We include the full source transaction contents
            .update(source_hash)
            // We also include the instructions hash, so the exact instructions can be proven
            .update(instructions_hash)
            .finalize();
        Ok(PreparedRoundUpdateTransactionV1 {
            encoded_instructions: manifest_encode(&prepared_instructions.inner.0)?,
            references: prepared_instructions.references,
            blobs: index_map_new(),
            summary: Summary {
                effective_length: prepared_instructions.summary.effective_length,
                total_bytes_hashed: prepared_instructions.summary.total_bytes_hashed,
                hash: round_update_hash,
            },
        })
    }
}

impl TransactionPayload for RoundUpdateTransactionV1 {
    type Versioned = FixedEnumVariant<{ TransactionDiscriminator::V1RoundUpdate as u8 }, Self>;
    type Prepared = PreparedRoundUpdateTransactionV1;
    type Raw = RawRoundUpdateTransactionV1;
}

pub struct PreparedRoundUpdateTransactionV1 {
    pub encoded_instructions: Vec<u8>,
    pub references: IndexSet<Reference>,
    pub blobs: IndexMap<Hash, Vec<u8>>,
    pub summary: Summary,
}

impl HasSummary for PreparedRoundUpdateTransactionV1 {
    fn get_summary(&self) -> &Summary {
        &self.summary
    }
}

define_raw_transaction_payload!(RawRoundUpdateTransactionV1);

impl TransactionPayloadPreparable for PreparedRoundUpdateTransactionV1 {
    type Raw = RawRoundUpdateTransactionV1;

    fn prepare_for_payload(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        let decoded = decoder
            .decode::<<RoundUpdateTransactionV1 as TransactionPayload>::Versioned>()?
            .fields;
        decoded.prepare()
    }
}

impl TransactionFullChildPreparable for PreparedRoundUpdateTransactionV1 {
    fn prepare_as_full_body_child(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        let decoded = decoder.decode::<RoundUpdateTransactionV1>()?;
        decoded.prepare()
    }
}

impl PreparedRoundUpdateTransactionV1 {
    pub fn get_executable(&self) -> Executable<'_> {
        Executable::new(
            &self.encoded_instructions,
            &self.references,
            &self.blobs,
            ExecutionContext {
                transaction_hash: self.summary.hash,
                payload_size: 0,
                auth_zone_params: AuthZoneParams {
                    initial_proofs: btreeset!(AuthAddresses::validator_role()),
                    virtual_resources: BTreeSet::new(),
                },
                fee_payment: FeePayment::NoFee,
                runtime_validations: vec![],
                pre_allocated_ids: indexset!(),
            },
        )
    }
}

define_wrapped_hash!(RoundUpdateTransactionHash);

impl HasRoundUpdateTransactionHash for PreparedRoundUpdateTransactionV1 {
    fn round_update_transaction_hash(&self) -> RoundUpdateTransactionHash {
        RoundUpdateTransactionHash::from_hash(self.summary.hash)
    }
}

pub trait HasRoundUpdateTransactionHash {
    fn round_update_transaction_hash(&self) -> RoundUpdateTransactionHash;
}
