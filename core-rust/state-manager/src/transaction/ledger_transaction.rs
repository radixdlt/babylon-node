use radix_engine_constants::TRANSACTION_HASHABLE_PAYLOAD_PREFIX;
use radix_engine_interface::api::node_modules::auth::AuthAddresses;
use radix_engine_interface::prelude::*;
use sbor::*;

use transaction::define_raw_transaction_payload;
use transaction::prelude::*;

use super::{
    HasRoundUpdateTransactionHash, PreparedRoundUpdateTransactionV1, RoundUpdateTransactionHash,
    RoundUpdateTransactionV1,
};

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub struct PayloadIdentifiers {
    pub ledger_payload_hash: LedgerTransactionHash,
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
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserTransactionIdentifiers<'a> {
    pub intent_hash: &'a IntentHash,
    pub signed_intent_hash: &'a SignedIntentHash,
    pub notarized_transaction_hash: &'a NotarizedTransactionHash,
}

impl TypedTransactionIdentifiers {
    pub fn user(&self) -> Option<UserTransactionIdentifiers> {
        match self {
            TypedTransactionIdentifiers::User {
                intent_hash,
                signed_intent_hash,
                notarized_transaction_hash,
            } => Some(UserTransactionIdentifiers {
                intent_hash,
                signed_intent_hash,
                notarized_transaction_hash,
            }),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ManifestCategorize, ManifestEncode, ManifestDecode)]
pub enum LedgerTransaction {
    #[sbor(discriminator(GENESIS_LEDGER_TRANSACTION_DISCRIMINATOR))]
    Genesis(Box<SystemTransactionV1>),
    #[sbor(discriminator(USER_V1_LEDGER_TRANSACTION_DISCRIMINATOR))]
    UserV1(Box<NotarizedTransactionV1>),
    #[sbor(discriminator(ROUND_UPDATE_V1_LEDGER_TRANSACTION_DISCRIMINATOR))]
    RoundUpdateV1(Box<RoundUpdateTransactionV1>),
}

const GENESIS_LEDGER_TRANSACTION_DISCRIMINATOR: u8 = 0;
const USER_V1_LEDGER_TRANSACTION_DISCRIMINATOR: u8 = 1;
const ROUND_UPDATE_V1_LEDGER_TRANSACTION_DISCRIMINATOR: u8 = 2;

define_raw_transaction_payload!(RawLedgerTransaction);

// We basically implement TransactionPayload manually for LedgerTransaction because it's not a struct
impl LedgerTransaction {
    pub fn to_raw(&self) -> Result<RawLedgerTransaction, EncodeError> {
        Ok(self.to_payload_bytes()?.into())
    }

    pub fn to_payload_bytes(&self) -> Result<Vec<u8>, EncodeError> {
        manifest_encode(&FixedEnumVariant::<
            { TransactionDiscriminator::V1Ledger as u8 },
            (&LedgerTransaction,),
        >::new((self,)))
    }

    pub fn from_raw(raw: &RawLedgerTransaction) -> Result<Self, DecodeError> {
        Self::from_payload_bytes(&raw.0)
    }

    pub fn from_raw_user(raw: &RawNotarizedTransaction) -> Result<Self, DecodeError> {
        Ok(LedgerTransaction::UserV1(Box::new(
            NotarizedTransactionV1::from_raw(raw)?,
        )))
    }

    pub fn from_payload_bytes(payload_bytes: &[u8]) -> Result<Self, DecodeError> {
        Ok(manifest_decode::<
            FixedEnumVariant<{ TransactionDiscriminator::V1Ledger as u8 }, (LedgerTransaction,)>,
        >(payload_bytes)?
        .into_fields()
        .0)
    }

    pub fn prepare(&self) -> Result<PreparedLedgerTransaction, PrepareError> {
        PreparedLedgerTransaction::prepare_from_payload(&self.to_payload_bytes()?)
    }
}

pub struct PreparedLedgerTransaction {
    pub inner: PreparedLedgerTransactionInner,
    pub summary: Summary,
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
            ledger_payload_hash: self.ledger_transaction_hash(),
            typed: match &self.inner {
                PreparedLedgerTransactionInner::Genesis(t) => {
                    TypedTransactionIdentifiers::Genesis {
                        system_transaction_hash: t.system_transaction_hash(),
                    }
                }
                PreparedLedgerTransactionInner::UserV1(t) => TypedTransactionIdentifiers::User {
                    intent_hash: t.intent_hash(),
                    signed_intent_hash: t.signed_intent_hash(),
                    notarized_transaction_hash: t.notarized_transaction_hash(),
                },
                PreparedLedgerTransactionInner::RoundUpdateV1(t) => {
                    TypedTransactionIdentifiers::RoundUpdateV1 {
                        round_update_hash: t.round_update_transaction_hash(),
                    }
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

#[derive(BasicCategorize)]
pub enum PreparedLedgerTransactionInner {
    #[sbor(discriminator(GENESIS_LEDGER_TRANSACTION_DISCRIMINATOR))]
    Genesis(Box<PreparedSystemTransactionV1>),
    #[sbor(discriminator(USER_V1_LEDGER_TRANSACTION_DISCRIMINATOR))]
    UserV1(Box<PreparedNotarizedTransactionV1>),
    #[sbor(discriminator(ROUND_UPDATE_V1_LEDGER_TRANSACTION_DISCRIMINATOR))]
    RoundUpdateV1(Box<PreparedRoundUpdateTransactionV1>),
}

impl PreparedLedgerTransactionInner {
    pub fn get_ledger_hash(&self) -> LedgerTransactionHash {
        LedgerTransactionHash::for_kind(self.get_discriminator(), &self.get_summary().hash)
    }
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

impl TransactionFullChildPreparable for PreparedLedgerTransactionInner {
    fn prepare_as_full_body_child(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        decoder.track_stack_depth_increase()?;
        let (discriminator, length) = decoder.read_enum_header()?;
        let prepared_inner = match discriminator {
            GENESIS_LEDGER_TRANSACTION_DISCRIMINATOR => {
                if length != 1 {
                    return Err(PrepareError::DecodeError(DecodeError::UnexpectedSize {
                        expected: 1,
                        actual: length,
                    }));
                }
                let prepared = PreparedSystemTransactionV1::prepare_as_full_body_child(decoder)?;
                PreparedLedgerTransactionInner::Genesis(Box::new(prepared))
            }
            USER_V1_LEDGER_TRANSACTION_DISCRIMINATOR => {
                if length != 1 {
                    return Err(PrepareError::DecodeError(DecodeError::UnexpectedSize {
                        expected: 1,
                        actual: length,
                    }));
                }
                let prepared = PreparedNotarizedTransactionV1::prepare_as_full_body_child(decoder)?;
                PreparedLedgerTransactionInner::UserV1(Box::new(prepared))
            }
            ROUND_UPDATE_V1_LEDGER_TRANSACTION_DISCRIMINATOR => {
                if length != 1 {
                    return Err(PrepareError::DecodeError(DecodeError::UnexpectedSize {
                        expected: 1,
                        actual: length,
                    }));
                }
                let prepared =
                    PreparedRoundUpdateTransactionV1::prepare_as_full_body_child(decoder)?;
                PreparedLedgerTransactionInner::RoundUpdateV1(Box::new(prepared))
            }
            _ => {
                return Err(PrepareError::DecodeError(
                    DecodeError::UnknownDiscriminator(discriminator),
                ));
            }
        };
        decoder.track_stack_depth_decrease()?;

        Ok(prepared_inner)
    }
}

impl TransactionPayloadPreparable for PreparedLedgerTransaction {
    type Raw = RawLedgerTransaction;

    fn prepare_for_payload(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        decoder.track_stack_depth_increase()?;
        decoder.read_expected_enum_variant_header(TransactionDiscriminator::V1Ledger as u8, 1)?;
        let inner = PreparedLedgerTransactionInner::prepare_as_full_body_child(decoder)?;
        decoder.track_stack_depth_decrease()?;

        let summary = Summary {
            effective_length: inner.get_summary().effective_length,
            total_bytes_hashed: inner.get_summary().total_bytes_hashed,
            hash: inner.get_ledger_hash().0,
        };
        Ok(Self { inner, summary })
    }
}

pub struct ValidatedLedgerTransaction {
    pub inner: ValidatedLedgerTransactionInner,
    pub summary: Summary,
}

/// Note - we don't allow System transactions here, because they are Genesis or Protocol Updates,
/// which are executed / inserted by the node, and not explicitly provided / validated from ledger sync
#[derive(BasicCategorize)]
pub enum ValidatedLedgerTransactionInner {
    #[sbor(discriminator(GENESIS_LEDGER_TRANSACTION_DISCRIMINATOR))]
    Genesis(Box<PreparedSystemTransactionV1>),
    #[sbor(discriminator(USER_V1_LEDGER_TRANSACTION_DISCRIMINATOR))]
    UserV1(Box<ValidatedNotarizedTransactionV1>),
    #[sbor(discriminator(ROUND_UPDATE_V1_LEDGER_TRANSACTION_DISCRIMINATOR))]
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
            ValidatedLedgerTransactionInner::Genesis(t) => {
                t.get_executable(btreeset!(AuthAddresses::system_role()))
            }
            ValidatedLedgerTransactionInner::UserV1(t) => t.get_executable(),
            ValidatedLedgerTransactionInner::RoundUpdateV1(t) => t.get_executable(),
        }
    }

    pub fn create_identifiers(&self) -> PayloadIdentifiers {
        PayloadIdentifiers {
            ledger_payload_hash: self.ledger_transaction_hash(),
            typed: match &self.inner {
                ValidatedLedgerTransactionInner::Genesis(t) => {
                    TypedTransactionIdentifiers::Genesis {
                        system_transaction_hash: t.system_transaction_hash(),
                    }
                }
                ValidatedLedgerTransactionInner::UserV1(t) => TypedTransactionIdentifiers::User {
                    intent_hash: t.intent_hash(),
                    signed_intent_hash: t.signed_intent_hash(),
                    notarized_transaction_hash: t.notarized_transaction_hash(),
                },
                ValidatedLedgerTransactionInner::RoundUpdateV1(t) => {
                    TypedTransactionIdentifiers::RoundUpdateV1 {
                        round_update_hash: t.round_update_transaction_hash(),
                    }
                }
            },
        }
    }
}

impl HasLedgerTransactionHash for ValidatedLedgerTransaction {
    fn ledger_transaction_hash(&self) -> LedgerTransactionHash {
        LedgerTransactionHash::from_hash(self.summary.hash)
    }
}

define_wrapped_hash!(LedgerTransactionHash);

impl LedgerTransactionHash {
    pub fn for_genesis(hash: &SystemTransactionHash) -> Self {
        Self::for_kind(GENESIS_LEDGER_TRANSACTION_DISCRIMINATOR, &hash.0)
    }

    pub fn for_user_v1(hash: &NotarizedTransactionHash) -> Self {
        Self::for_kind(USER_V1_LEDGER_TRANSACTION_DISCRIMINATOR, &hash.0)
    }

    pub fn for_round_update_v1(hash: &RoundUpdateTransactionHash) -> Self {
        Self::for_kind(ROUND_UPDATE_V1_LEDGER_TRANSACTION_DISCRIMINATOR, &hash.0)
    }

    fn for_kind(discriminator: u8, inner: &Hash) -> Self {
        Self(
            HashAccumulator::new()
                .update([
                    TRANSACTION_HASHABLE_PAYLOAD_PREFIX,
                    TransactionDiscriminator::V1Ledger as u8,
                    discriminator,
                ])
                .update(inner.as_slice())
                .finalize(),
        )
    }
}

pub trait HasLedgerTransactionHash {
    fn ledger_transaction_hash(&self) -> LedgerTransactionHash;
}

impl HasLedgerTransactionHash for PreparedLedgerTransaction {
    fn ledger_transaction_hash(&self) -> LedgerTransactionHash {
        LedgerTransactionHash::from_hash(self.summary.hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn v1_ledger_transaction_structure() {
        let sig_1_private_key = Secp256k1PrivateKey::from_u64(1).unwrap();
        let sig_2_private_key = Ed25519PrivateKey::from_u64(2).unwrap();
        let notary_private_key = Ed25519PrivateKey::from_u64(3).unwrap();

        let notarized = TransactionBuilder::new()
            .header(TransactionHeaderV1 {
                network_id: 21,
                start_epoch_inclusive: Epoch::of(0),
                end_epoch_exclusive: Epoch::of(100),
                nonce: 0,
                notary_public_key: notary_private_key.public_key().into(),
                notary_is_signatory: true,
                tip_percentage: 0,
            })
            .manifest(ManifestBuilder::new().clear_auth_zone().build())
            .sign(&sig_1_private_key)
            .sign(&sig_2_private_key)
            .notarize(&notary_private_key)
            .build();

        let prepared_notarized = notarized.prepare().expect("Notarized can be prepared");

        let ledger = LedgerTransaction::UserV1(Box::new(notarized));
        let ledger_transaction_bytes = ledger.to_payload_bytes().expect("Can be encoded");
        LedgerTransaction::from_payload_bytes(&ledger_transaction_bytes).expect("Can be decoded");
        let prepared_ledger_transaction =
            PreparedLedgerTransaction::prepare_from_payload(&ledger_transaction_bytes)
                .expect("Can be prepared");

        let expected_intent_hash = LedgerTransactionHash::from_hash(hash(
            [
                [
                    TRANSACTION_HASHABLE_PAYLOAD_PREFIX,
                    TransactionDiscriminator::V1Ledger as u8,
                    USER_V1_LEDGER_TRANSACTION_DISCRIMINATOR,
                ]
                .as_slice(),
                prepared_notarized.notarized_transaction_hash().0.as_slice(),
            ]
            .concat(),
        ));
        assert_eq!(
            prepared_ledger_transaction.ledger_transaction_hash(),
            expected_intent_hash
        );
        assert_eq!(
            LedgerTransactionHash::for_user_v1(&prepared_notarized.notarized_transaction_hash()),
            expected_intent_hash
        );
    }
}
