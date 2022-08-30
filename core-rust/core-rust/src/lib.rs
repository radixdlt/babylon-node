/// A workaround for including the symbols defined in state_manager / core_api_server
/// in the output library file. See: https://github.com/rust-lang/rfcs/issues/2771
/// I truly have no idea why this works, but it does.
#[no_mangle]
fn export_extern_functions() {
    // state-manager
    state_manager::jni::mempool::export_extern_functions();
    state_manager::jni::state_computer::export_extern_functions();
    state_manager::jni::state_manager::export_extern_functions();
    state_manager::jni::transaction_builder::export_extern_functions();
    state_manager::jni::transaction_store::export_extern_functions();
    state_manager::jni::vertex_store_recovery::export_extern_functions();

    // core-api-server
    core_api_server::jni::export_extern_functions();
}
