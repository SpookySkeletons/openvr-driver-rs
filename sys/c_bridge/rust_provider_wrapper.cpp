#include "rust_provider_bridge.h"
#include "../openvr/headers/openvr_driver.h"
#include <iostream>

// C++ wrapper class that implements the OpenVR interface
class RustServerTrackedDeviceProvider : public vr::IServerTrackedDeviceProvider {
private:
    RustProviderHandle* rust_handle;

public:
    RustServerTrackedDeviceProvider() {
        std::cout << "RustServerTrackedDeviceProvider: Creating..." << std::endl;
        rust_handle = rust_provider_create();
        std::cout << "RustServerTrackedDeviceProvider: Created, handle = " << rust_handle << std::endl;
    }

    virtual ~RustServerTrackedDeviceProvider() {
        std::cout << "RustServerTrackedDeviceProvider: Destroying..." << std::endl;
        if (rust_handle) {
            rust_provider_destroy(rust_handle);
            rust_handle = nullptr;
        }
    }

    // For now, implement minimal required methods
    virtual vr::EVRInitError Init(vr::IVRDriverContext* pDriverContext) override {
        std::cout << "RustServerTrackedDeviceProvider::Init called!" << std::endl;
        if (rust_handle && rust_provider_init(rust_handle, pDriverContext) == 0) {
            std::cout << "RustServerTrackedDeviceProvider::Init - Rust provider initialized successfully!" << std::endl;
            return vr::VRInitError_None;
        } else {
            std::cout << "RustServerTrackedDeviceProvider::Init - Failed to initialize Rust provider!" << std::endl;
            return vr::VRInitError_Init_InitCanceledByUser;
        }
    }

    virtual void Cleanup() override {
        std::cout << "RustServerTrackedDeviceProvider::Cleanup called!" << std::endl;
        if (rust_handle) {
            rust_provider_cleanup(rust_handle);
        }
    }

    virtual const char* const* GetInterfaceVersions() override {
        static const char* versions[] = { "IServerTrackedDeviceProvider_004", nullptr };
        return versions;
    }

    virtual void RunFrame() override {
        if (rust_handle) {
            rust_provider_run_frame(rust_handle);
        }
    }

    virtual bool ShouldBlockStandbyMode() override {
        if (rust_handle) {
            return rust_provider_should_block_standby(rust_handle) != 0;
        }
        return false;
    }

    virtual void EnterStandby() override {
        if (rust_handle) {
            rust_provider_enter_standby(rust_handle);
        }
    }
    
    virtual void LeaveStandby() override {
        if (rust_handle) {
            rust_provider_leave_standby(rust_handle);
        }
    }
};

// Factory function for creating the wrapper
extern "C" void* create_rust_server_provider() {
    std::cout << "create_rust_server_provider: Creating wrapper..." << std::endl;
    return new RustServerTrackedDeviceProvider();
}

extern "C" void* driver_context_get_generic_interface(void* context, const char* interface_version, int* error) {
    if (!context || !interface_version) {
        if (error) *error = static_cast<int>(vr::VRInitError_Init_InvalidInterface);
        return nullptr;
    }

    vr::IVRDriverContext* driver_context = static_cast<vr::IVRDriverContext*>(context);
    vr::EVRInitError vr_error = vr::VRInitError_None;

    void* result = driver_context->GetGenericInterface(interface_version, &vr_error);

    if (error) *error = static_cast<int>(vr_error);
    return result;
}

extern "C" uint64_t driver_context_get_driver_handle(void* context) {
    if (!context) return 0;

    vr::IVRDriverContext* driver_context = static_cast<vr::IVRDriverContext*>(context);
    return driver_context->GetDriverHandle();
}
