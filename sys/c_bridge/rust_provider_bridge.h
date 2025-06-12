#ifndef RUST_PROVIDER_BRIDGE_H
#define RUST_PROVIDER_BRIDGE_H

#ifdef __cplusplus
extern "C" {
#endif

// Opaque handle to Rust provider
typedef struct RustProviderHandle RustProviderHandle;

// Basic C functions that Rust will implement
RustProviderHandle* rust_provider_create(void);
void rust_provider_destroy(RustProviderHandle* handle);

// Factory function for creating the C++ wrapper
void* create_rust_server_provider(void);

// Driver functions
int rust_provider_init(RustProviderHandle* handle, void* driver_context);
void rust_provider_cleanup(RustProviderHandle* handle);
void rust_provider_run_frame(RustProviderHandle* handle);
int rust_provider_should_block_standby(RustProviderHandle* handle);
void rust_provider_enter_standby(RustProviderHandle* handle);
void rust_provider_leave_standby(RustProviderHandle* handle);

#ifdef __cplusplus
}
#endif

#endif // RUST_PROVIDER_BRIDGE_H
