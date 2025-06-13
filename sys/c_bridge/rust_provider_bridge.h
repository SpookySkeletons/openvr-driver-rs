#ifndef RUST_PROVIDER_BRIDGE_H
#define RUST_PROVIDER_BRIDGE_H

#include "cstdint"

#ifdef __cplusplus
extern "C" {
#endif

// Opaque handle to Rust provider
typedef struct RustProviderHandle RustProviderHandle;

// Basic C functions that Rust will implement
RustProviderHandle *rust_provider_create(void);
void rust_provider_destroy(RustProviderHandle *handle);

// Factory function for creating the C++ wrapper
void *create_rust_server_provider(void);

void *driver_context_get_generic_interface(void *context,
                                           const char *interface_version,
                                           int *error);
uint64_t driver_context_get_driver_handle(void *context);

// Driver functions
int rust_provider_init(RustProviderHandle *handle, void *driver_context);
void rust_provider_cleanup(RustProviderHandle *handle);
void rust_provider_run_frame(RustProviderHandle *handle);
int rust_provider_should_block_standby(RustProviderHandle *handle);
void rust_provider_enter_standby(RustProviderHandle* handle);
void rust_provider_leave_standby(RustProviderHandle* handle);

// Bridge function for TrackedDeviceAdded
bool server_driver_host_tracked_device_added(
    void* host_ptr,
    const char* serial_number,
    int device_class,
    void* device_driver_ptr
);

// Generic device bridge functions - work with any device type
typedef struct RustDeviceBridge RustDeviceBridge;

void rust_device_destroy(RustDeviceBridge* handle);
int rust_device_activate(RustDeviceBridge* handle, uint32_t device_id);
void rust_device_deactivate(RustDeviceBridge* handle);
void rust_device_run_frame(RustDeviceBridge* handle);
void rust_device_enter_standby(RustDeviceBridge* handle);

// Factory function for creating C++ device wrapper from generic bridge
void* create_rust_device_wrapper(RustDeviceBridge* rust_device_bridge);

#ifdef __cplusplus
}
#endif

#endif // RUST_PROVIDER_BRIDGE_H
