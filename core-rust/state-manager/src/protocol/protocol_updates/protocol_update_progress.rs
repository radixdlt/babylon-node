use crate::prelude::*;

pub enum ProtocolUpdateProgress {
    UpdateInitiatedButNothingCommitted {
        protocol_version_name: ProtocolVersionName,
    },
    UpdateInProgress {
        protocol_version_name: ProtocolVersionName,
        last_batch_group_index: usize,
        last_batch_index: usize,
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
            return ProtocolUpdateProgress::UpdateInitiatedButNothingCommitted {
                protocol_version_name: ProtocolVersionName::babylon(),
            };
        };

        if let Some(latest_proof_protocol_version) =
            &latest_proof.ledger_header.next_protocol_version
        {
            return ProtocolUpdateProgress::UpdateInitiatedButNothingCommitted {
                protocol_version_name: latest_proof_protocol_version.clone(),
            };
        }

        match latest_proof.origin {
            LedgerProofOrigin::Consensus { .. } => ProtocolUpdateProgress::NotUpdating,
            LedgerProofOrigin::ProtocolUpdate {
                protocol_version_name,
                config_hash: _,
                batch_name: _,
                batch_group_index,
                batch_group_name: _,
                batch_index,
                is_end_of_update,
            } => {
                if is_end_of_update {
                    ProtocolUpdateProgress::NotUpdating
                } else {
                    ProtocolUpdateProgress::UpdateInProgress {
                        protocol_version_name: protocol_version_name.clone(),
                        last_batch_group_index: batch_group_index,
                        last_batch_index: batch_index,
                    }
                }
            }
        }
    }
}
