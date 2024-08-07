use crate::engine_prelude::*;

use crate::protocol::*;

use crate::consensus::traits::*;

use crate::LedgerProofOrigin;

pub enum ProtocolUpdateProgress {
    UpdateInitiatedButNothingCommitted {
        protocol_version_name: ProtocolVersionName,
    },
    UpdateInProgress {
        protocol_version_name: ProtocolVersionName,
        last_batch_idx: u32,
    },
    /// This means that the last proof contains no notion of a protocol update,
    /// which in practice almost always means that it has already executed in full.
    /// But we leave this interpretation to the caller,
    /// so here we just call it "not updating".
    NotUpdating,
}

impl ProtocolUpdateProgress {
    /// Inspects the database in order to resolve the current Protocol Update progress.
    pub fn resolve(database: &impl QueryableProofStore) -> Self {
        let Some(latest_proof) = database.get_latest_proof() else {
            return ProtocolUpdateProgress::NotUpdating;
        };

        match &latest_proof.origin {
            LedgerProofOrigin::Genesis { .. } => ProtocolUpdateProgress::NotUpdating,
            LedgerProofOrigin::Consensus { .. } => {
                if let Some(latest_proof_protocol_version) =
                    latest_proof.ledger_header.next_protocol_version
                {
                    ProtocolUpdateProgress::UpdateInitiatedButNothingCommitted {
                        protocol_version_name: ProtocolVersionName::of_unchecked(
                            latest_proof_protocol_version,
                        ),
                    }
                } else {
                    ProtocolUpdateProgress::NotUpdating
                }
            }
            LedgerProofOrigin::ProtocolUpdate {
                protocol_version_name,
                batch_idx,
            } => ProtocolUpdateProgress::UpdateInProgress {
                protocol_version_name: protocol_version_name.clone(),
                last_batch_idx: *batch_idx,
            },
        }
    }
}
