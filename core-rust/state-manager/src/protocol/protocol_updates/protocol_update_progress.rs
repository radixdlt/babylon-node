use crate::engine_prelude::*;

use crate::protocol::*;

use crate::traits::*;

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

    /// Returns the new instance scoped at the given protocol version.
    /// This means that any other version's Protocol Update will be considered "not updating".
    pub fn scoped_on(self, specific_protocol_version: &ProtocolVersionName) -> Self {
        match &self {
            ProtocolUpdateProgress::UpdateInitiatedButNothingCommitted {
                protocol_version_name,
            }
            | ProtocolUpdateProgress::UpdateInProgress {
                protocol_version_name,
                ..
            } if protocol_version_name == specific_protocol_version => self,
            _ => ProtocolUpdateProgress::NotUpdating,
        }
    }

    /// Returns an index of the next batch to be executed for an active Protocol Update (or [`None`]
    /// if not updating).
    pub fn next_batch_idx(&self) -> Option<u32> {
        match self {
            ProtocolUpdateProgress::UpdateInitiatedButNothingCommitted { .. } => Some(0),
            ProtocolUpdateProgress::UpdateInProgress { last_batch_idx, .. } => {
                Some(last_batch_idx.checked_add(1).unwrap())
            }
            ProtocolUpdateProgress::NotUpdating => None,
        }
    }
}
