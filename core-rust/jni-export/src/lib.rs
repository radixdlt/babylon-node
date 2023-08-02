/// A workaround for including the symbols defined in state_manager / core_api_server
/// in the output library file. See: https://github.com/rust-lang/rfcs/issues/2771
/// I truly have no idea why this works, but it does.
#[no_mangle]
fn export_extern_functions() {
    // node-common
    node_common::jni::addressing::export_extern_functions();
    node_common::jni::scrypto_constants::export_extern_functions();

    // state-manager
    state_manager::jni::mempool::export_extern_functions();
    state_manager::jni::state_computer::export_extern_functions();
    state_manager::jni::rust_global_context::export_extern_functions();
    state_manager::jni::transaction_preparer::export_extern_functions();
    state_manager::jni::transaction_store::export_extern_functions();
    state_manager::jni::vertex_store_recovery::export_extern_functions();
    state_manager::jni::test_state_reader::export_extern_functions();

    // core-api-server
    core_api_server::jni::export_extern_functions();
}
