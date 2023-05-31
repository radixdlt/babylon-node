use radix_engine_constants::TRANSACTION_HASHABLE_PAYLOAD_PREFIX;
use radix_engine_interface::api::node_modules::auth::AuthAddresses;
use radix_engine_interface::prelude::*;
use sbor::*;

use transaction::prelude::*;
use transaction::define_raw_transaction_payload;

use super::{RoundUpdateTransactionV1, PreparedRoundUpdateTransactionV1, RoundUpdateTransactionHash, HasRoundUpdateTransactionHash};

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub struct PayloadIdentifiers {
    pub ledger_payload_hash: LedgerPayloadHash,
    pub typed: TypedTransactionIdentifiers,
}

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub enum TypedTransactionIdentifiers {
    Genesis {
        system_transaction_hash: SystemTransactionHash,
    },
    User {
        intent_hash: IntentHash,
        signed_intent_hash: SignedIntentHash,
        notarized_transaction_hash: NotarizedTransactionHash,
    },
    RoundUpdateV1 {
        round_update_hash: RoundUpdateTransactionHash,
    }
}

impl TypedTransactionIdentifiers {
    pub fn user(&self) -> Option<(&IntentHash, &SignedIntentHash, &NotarizedTransactionHash)> {
        match self {
            TypedTransactionIdentifiers::User {
                intent_hash,
                signed_intent_hash,
                notarized_transaction_hash
            } => Some((intent_hash, signed_intent_hash, notarized_transaction_hash)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ManifestCategorize, ManifestEncode, ManifestDecode)]
pub enum LedgerTransaction {
    Genesis(Box<SystemTransactionV1>),
    UserV1(Box<NotarizedTransactionV1>),
    RoundUpdateV1(Box<RoundUpdateTransactionV1>),
}

define_raw_transaction_payload!(RawLedgerTransaction);

// We basically implement TransactionPayload manually for LedgerTransaction because it's not a struct
impl LedgerTransaction {
    pub fn to_raw(&self) -> Result<RawLedgerTransaction, EncodeError> {
        Ok(self.to_payload_bytes()?.into())
    }

    pub fn to_payload_bytes(&self) -> Result<Vec<u8>, EncodeError> {
        manifest_encode(&FixedEnumVariant::<{ TransactionDiscriminator::V1Ledger as u8 }, (&LedgerTransaction,)>::new((self,)))
    }

    pub fn from_raw(raw: &RawLedgerTransaction) -> Result<Self, DecodeError> {
        Self::from_payload_bytes(&raw.0)
    }

    pub fn from_raw_user(raw: &RawNotarizedTransaction) -> Result<Self, DecodeError> {
        Ok(LedgerTransaction::UserV1(Box::new(NotarizedTransactionV1::from_raw(raw)?)))
    }

    pub fn from_payload_bytes(payload_bytes: &[u8]) -> Result<Self, DecodeError> {
        Ok(manifest_decode::<FixedEnumVariant::<{ TransactionDiscriminator::V1Ledger as u8 }, (LedgerTransaction,)>>(payload_bytes)?.into_fields().0)
    }

    pub fn prepare(&self) -> Result<PreparedLedgerTransaction, PrepareError> {
        PreparedLedgerTransaction::prepare_from_payload(
            &self.to_payload_bytes()?,
        )
    }
}

pub struct PreparedLedgerTransaction {
    pub inner: PreparedLedgerTransactionInner,
    pub summary: Summary,
    pub legacy_ledger_payload_hash: LegacyLedgerPayloadHash,
}

impl PreparedLedgerTransaction {
    pub fn into_user(self) -> Option<Box<PreparedNotarizedTransactionV1>> {
        match self.inner {
            PreparedLedgerTransactionInner::UserV1(t) => Some(t),
            _ => None,
        }
    }

    pub fn as_user(&self) -> Option<&PreparedNotarizedTransactionV1> {
        match &self.inner {
            PreparedLedgerTransactionInner::UserV1(t) => Some(t.as_ref()),
            _ => None,
        }
    }

    pub fn into_genesis(self) -> Option<Box<PreparedSystemTransactionV1>> {
        match self.inner {
            PreparedLedgerTransactionInner::Genesis(t) => Some(t),
            _ => None,
        }
    }

    pub fn create_identifiers(&self) -> PayloadIdentifiers {
        PayloadIdentifiers {
            ledger_payload_hash: self.ledger_payload_hash(),
            typed: match &self.inner {
                PreparedLedgerTransactionInner::Genesis(t) => TypedTransactionIdentifiers::Genesis {
                    system_transaction_hash: t.system_transaction_hash()
                },
                PreparedLedgerTransactionInner::UserV1(t) => TypedTransactionIdentifiers::User {
                    intent_hash: t.intent_hash(),
                    signed_intent_hash: t.signed_intent_hash(),
                    notarized_transaction_hash: t.notarized_transaction_hash(),
                },
                PreparedLedgerTransactionInner::RoundUpdateV1(t) => TypedTransactionIdentifiers::RoundUpdateV1 {
                    round_update_hash: t.round_update_transaction_hash(),
                }
            },
        }
    }
}

impl HasSummary for PreparedLedgerTransaction {
    fn get_summary(&self) -> &Summary {
        &self.summary
    }
}

pub enum PreparedLedgerTransactionInner {
    Genesis(Box<PreparedSystemTransactionV1>),
    UserV1(Box<PreparedNotarizedTransactionV1>),
    RoundUpdateV1(Box<PreparedRoundUpdateTransactionV1>),
}

impl HasSummary for PreparedLedgerTransactionInner {
    fn get_summary(&self) -> &Summary {
        match self {
            Self::Genesis(t) => t.get_summary(),
            Self::UserV1(t) => t.get_summary(),
            Self::RoundUpdateV1(t) => t.get_summary(),
        }
    }
}

impl TransactionPayloadPreparable for PreparedLedgerTransaction {
    type Raw = RawLedgerTransaction;

    fn prepare_for_payload(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        decoder.track_stack_depth_increase()?;
        let (discriminator, length) = decoder.read_enum_header()?;
        let prepared_inner = match discriminator {
            0 => {
                if length != 1 {
                    return Err(PrepareError::DecodeError(DecodeError::UnexpectedSize { expected: 1, actual: length }))
                }
                let prepared = PreparedSystemTransactionV1::prepare_as_full_body_child(decoder)?;
                PreparedLedgerTransactionInner::Genesis(Box::new(prepared))
            },
            1 => {
                if length != 1 {
                    return Err(PrepareError::DecodeError(DecodeError::UnexpectedSize { expected: 1, actual: length }))
                }
                let prepared = PreparedNotarizedTransactionV1::prepare_as_full_body_child(decoder)?;
                PreparedLedgerTransactionInner::UserV1(Box::new(prepared))
            },
            2 => {
                if length != 1 {
                    return Err(PrepareError::DecodeError(DecodeError::UnexpectedSize { expected: 1, actual: length }))
                }
                let prepared = PreparedRoundUpdateTransactionV1::prepare_as_full_body_child(decoder)?;
                PreparedLedgerTransactionInner::RoundUpdateV1(Box::new(prepared))
            },
            _ => {
                return Err(PrepareError::DecodeError(DecodeError::UnknownDiscriminator(discriminator)));
            }
        };

        let ledger_payload_hash = HashAccumulator::new()
            .update([TRANSACTION_HASHABLE_PAYLOAD_PREFIX, TransactionDiscriminator::V1Ledger as u8])
            .update([discriminator])
            .update(prepared_inner.get_summary().hash.as_slice())
            .finalize();

        let summary = Summary {
            effective_length: prepared_inner.get_summary().effective_length,
            total_bytes_hashed: prepared_inner.get_summary().total_bytes_hashed,
            hash: ledger_payload_hash,
        };

        decoder.track_stack_depth_decrease()?;
        let end_offset = decoder.get_offset();
        
        Ok(Self {
            inner: prepared_inner,
            summary,
            // TODO - remove this when we change the legacy payload hash behaviour for ledger sync
            // Note - we assume that the payload started at 0 and ends at end_offset
            legacy_ledger_payload_hash: hash(decoder.get_slice(0, end_offset)).into()
        })
    }
}

pub struct ValidatedLedgerTransaction {
    pub inner: ValidatedLedgerTransactionInner,
    pub summary: Summary,
    pub legacy_ledger_payload_hash: LegacyLedgerPayloadHash,
}

/// Note - we don't allow System transactions here, because they are Genesis or Protocol Updates,
/// which are executed / inserted by the node, and not explicitly provided / validated from ledger sync
pub enum ValidatedLedgerTransactionInner {
    Genesis(Box<PreparedSystemTransactionV1>),
    UserV1(Box<ValidatedNotarizedTransactionV1>),
    RoundUpdateV1(Box<PreparedRoundUpdateTransactionV1>),
}

impl ValidatedLedgerTransaction {
    pub fn intent_hash_if_user(&self) -> Option<IntentHash> {
        match &self.inner {
            ValidatedLedgerTransactionInner::Genesis(_) => None,
            ValidatedLedgerTransactionInner::UserV1(t) => Some(t.intent_hash()),
            ValidatedLedgerTransactionInner::RoundUpdateV1(_) => None,
        }
    }

    pub fn get_executable(&self) -> Executable<'_> {
        match &self.inner {
            ValidatedLedgerTransactionInner::Genesis(t) => t.get_executable(btreeset!(AuthAddresses::system_role())),
            ValidatedLedgerTransactionInner::UserV1(t) => t.get_executable(),
            ValidatedLedgerTransactionInner::RoundUpdateV1(t) => t.get_executable(),
        }
    }

    pub fn create_identifiers(&self) -> PayloadIdentifiers {
        PayloadIdentifiers {
            ledger_payload_hash: self.ledger_payload_hash(),
            typed: match &self.inner {
                ValidatedLedgerTransactionInner::Genesis(t) => TypedTransactionIdentifiers::Genesis {
                    system_transaction_hash: t.system_transaction_hash(),
                },
                ValidatedLedgerTransactionInner::UserV1(t) => TypedTransactionIdentifiers::User {
                    intent_hash: t.intent_hash(),
                    signed_intent_hash: t.signed_intent_hash(),
                    notarized_transaction_hash: t.notarized_transaction_hash(),
                },
                ValidatedLedgerTransactionInner::RoundUpdateV1(t) => TypedTransactionIdentifiers::RoundUpdateV1 {
                    round_update_hash: t.round_update_transaction_hash(),
                },
            },
        }
    }
}

impl HasLedgerPayloadHash for ValidatedLedgerTransaction {
    fn ledger_payload_hash(&self) -> LedgerPayloadHash {
        LedgerPayloadHash::from_hash(self.summary.hash)
    }
}

impl HasLegacyLedgerPayloadHash for ValidatedLedgerTransaction {
    fn legacy_ledger_payload_hash(&self) -> LegacyLedgerPayloadHash {
        self.legacy_ledger_payload_hash
    }
}

// A hash of the whole payload, for use by the accumulator
// TODO: Remove
define_wrapped_hash!(LegacyLedgerPayloadHash);

pub trait HasLegacyLedgerPayloadHash {
    fn legacy_ledger_payload_hash(&self) -> LegacyLedgerPayloadHash;
}

impl HasLegacyLedgerPayloadHash for PreparedLedgerTransaction {
    fn legacy_ledger_payload_hash(&self) -> LegacyLedgerPayloadHash {
        self.legacy_ledger_payload_hash
    }
}

define_wrapped_hash!(LedgerPayloadHash);

pub trait HasLedgerPayloadHash {
    fn ledger_payload_hash(&self) -> LedgerPayloadHash;
}

impl HasLedgerPayloadHash for PreparedLedgerTransaction {
    fn ledger_payload_hash(&self) -> LedgerPayloadHash {
        LedgerPayloadHash::from_hash(self.summary.hash)
    }
}

#[cfg(test)]
mod tests {
    use std::unimplemented;

    #[test]
    pub fn v1_ledger_transaction_structure() {
        unimplemented!()
    }
}