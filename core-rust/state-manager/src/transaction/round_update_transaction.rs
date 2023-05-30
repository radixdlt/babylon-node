use radix_engine::types::*;

use radix_engine_interface::api::node_modules::auth::AuthAddresses;
use radix_engine_interface::blueprints::consensus_manager::*;
use transaction::{model::*, define_raw_transaction_payload};
use sbor::FixedEnumVariant;

#[derive(Debug, Clone, Categorize, Encode, Decode, PartialEq, Eq)]
pub struct RoundUpdateTransactionV1 {
    pub proposer_timestamp_ms: i64,
    pub epoch: Epoch,
    pub round: Round,
    pub leader_proposal_history: LeaderProposalHistory,
}

impl RoundUpdateTransactionV1 {
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
        let hash = HashAccumulator::new()
            .update([TRANSACTION_HASHABLE_PAYLOAD_PREFIX, TransactionDiscriminator::V1RoundUpdate as u8])
            .update(format!("RoundChange({},{})", self.epoch.number(), self.round.number()))
            .finalize();
        let prepared_instructions = InstructionsV1(self.create_instructions()).prepare_partial()?;
        Ok(PreparedRoundUpdateTransactionV1 {
            encoded_instructions: manifest_encode(&prepared_instructions.inner.0)?,
            references: prepared_instructions.references,
            blobs: index_map_new(),
            summary: Summary { effective_length: 0, total_bytes_hashed: 0, hash },
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
        let decoded = decoder.decode::<<RoundUpdateTransactionV1 as TransactionPayload>::Versioned>()?.fields;
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