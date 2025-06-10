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
        // TODO: Call rust_provider_init() in next piece
        return vr::VRInitError_None;
    }

    virtual void Cleanup() override {
        std::cout << "RustServerTrackedDeviceProvider::Cleanup called!" << std::endl;
        // TODO: Call rust_provider_cleanup() in next piece
    }

    virtual const char* const* GetInterfaceVersions() override {
        static const char* versions[] = { "IServerTrackedDeviceProvider_004", nullptr };
        return versions;
    }

    virtual void RunFrame() override {
        // TODO: Call rust_provider_run_frame() in next piece
    }

    virtual bool ShouldBlockStandbyMode() override {
        return false;
    }

    virtual void EnterStandby() override {}
    virtual void LeaveStandby() override {}
};

// Factory function for creating the wrapper
extern "C" void* create_rust_server_provider() {
    std::cout << "create_rust_server_provider: Creating wrapper..." << std::endl;
    return new RustServerTrackedDeviceProvider();
}
