// This file contains the protocol update logic for specific protocol versions

// TODO(protocol-updates): stub; implement me
pub enum ProtocolUpdateAction {}

/// Returns a list of _init actions_ (i.e. actions that should be run
/// before processing any other transactions using the new protocol version).
/// Returns the actions starting at the given checkpoint ID up to
/// the next checkpoint or None if there are no more actions.
// TODO(protocol-updates): stub; implement me
pub fn protocol_init(
    _protocol_version_name: String,
    _checkpoint_id: u32,
) -> Option<Vec<ProtocolUpdateAction>> {
    None
}
