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

// C++ wrapper class that implements the device interface
class RustTrackedDeviceServerDriver : public vr::ITrackedDeviceServerDriver {
private:
    RustDeviceHandle* rust_device;
    uint32_t device_id;

public:
    RustTrackedDeviceServerDriver(RustDeviceHandle* device) : rust_device(device), device_id(0) {
        std::cout << "RustTrackedDeviceServerDriver: Creating device wrapper for handle " << device << std::endl;
    }

    virtual ~RustTrackedDeviceServerDriver() {
        std::cout << "RustTrackedDeviceServerDriver: Destroying device wrapper" << std::endl;
        if (rust_device) {
            rust_device_destroy(rust_device);
            rust_device = nullptr;
        }
    }

    // Called when OpenVR wants to start using this device
    virtual vr::EVRInitError Activate(uint32_t unObjectId) override {
        std::cout << "RustTrackedDeviceServerDriver::Activate called with device ID " << unObjectId << std::endl;
        device_id = unObjectId;
        
        if (rust_device && rust_device_activate(rust_device, unObjectId) == 0) {
            std::cout << "RustTrackedDeviceServerDriver::Activate - Device activated successfully!" << std::endl;
            return vr::VRInitError_None;
        } else {
            std::cout << "RustTrackedDeviceServerDriver::Activate - Failed to activate device!" << std::endl;
            return vr::VRInitError_Init_InitCanceledByUser;
        }
    }

    // Called when OpenVR is done with this device
    virtual void Deactivate() override {
        std::cout << "RustTrackedDeviceServerDriver::Deactivate called!" << std::endl;
        if (rust_device) {
            rust_device_deactivate(rust_device);
        }
    }

    // Called when device should enter standby
    virtual void EnterStandby() override {
        if (rust_device) {
            rust_device_enter_standby(rust_device);
        }
    }

    // Get component interface - for now return nullptr
    virtual void* GetComponent(const char* pchComponentNameAndVersion) override {
        std::cout << "RustTrackedDeviceServerDriver::GetComponent called for: " << pchComponentNameAndVersion << std::endl;
        return nullptr;
    }

    // Debug request handler
    virtual void DebugRequest(const char* pchRequest, char* pchResponseBuffer, uint32_t unResponseBufferSize) override {
        if (unResponseBufferSize >= 1) {
            pchResponseBuffer[0] = 0;
        }
    }

    // Get current pose - for now return a simple identity pose
    virtual vr::DriverPose_t GetPose() override {
        vr::DriverPose_t pose = { 0 };
        pose.poseIsValid = true;
        pose.result = vr::TrackingResult_Running_OK;
        pose.deviceIsConnected = true;
        
        // Identity transform
        pose.qWorldFromDriverRotation.w = 1.0;
        pose.qWorldFromDriverRotation.x = 0.0;
        pose.qWorldFromDriverRotation.y = 0.0;
        pose.qWorldFromDriverRotation.z = 0.0;
        
        pose.vecWorldFromDriverTranslation[0] = 0.0;
        pose.vecWorldFromDriverTranslation[1] = 0.0;
        pose.vecWorldFromDriverTranslation[2] = 0.0;

        pose.qDriverFromHeadRotation.w = 1.0;
        pose.qDriverFromHeadRotation.x = 0.0;
        pose.qDriverFromHeadRotation.y = 0.0;
        pose.qDriverFromHeadRotation.z = 0.0;
        
        pose.vecDriverFromHeadTranslation[0] = 0.0;
        pose.vecDriverFromHeadTranslation[1] = 0.0;
        pose.vecDriverFromHeadTranslation[2] = 0.0;

        // Current pose in driver space
        pose.qRotation.w = 1.0;
        pose.qRotation.x = 0.0;
        pose.qRotation.y = 0.0;
        pose.qRotation.z = 0.0;
        
        pose.vecPosition[0] = 0.0;
        pose.vecPosition[1] = 0.0;
        pose.vecPosition[2] = 0.0;

        return pose;
    }
};

extern "C" bool server_driver_host_tracked_device_added(
    void* host_ptr,
    const char* serial_number, 
    int device_class,
    void* device_driver_ptr
) {
    if (!host_ptr || !serial_number || !device_driver_ptr) {
        std::cout << "server_driver_host_tracked_device_added: Invalid parameters" << std::endl;
        return false;
    }
    
    vr::IVRServerDriverHost* host = static_cast<vr::IVRServerDriverHost*>(host_ptr);
    vr::ETrackedDeviceClass vr_device_class = static_cast<vr::ETrackedDeviceClass>(device_class);
    vr::ITrackedDeviceServerDriver* device = static_cast<vr::ITrackedDeviceServerDriver*>(device_driver_ptr);
    
    std::cout << "server_driver_host_tracked_device_added: Calling TrackedDeviceAdded for '" << serial_number << "'" << std::endl;
    bool result = host->TrackedDeviceAdded(serial_number, vr_device_class, device);
    std::cout << "server_driver_host_tracked_device_added: Result = " << (result ? "success" : "failed") << std::endl;
    
    return result;
}

// Factory function for creating C++ device wrapper
extern "C" void* create_rust_device_wrapper(RustDeviceHandle* rust_device) {
    std::cout << "create_rust_device_wrapper: Creating C++ wrapper for Rust device " << rust_device << std::endl;
    if (!rust_device) {
        std::cout << "create_rust_device_wrapper: Invalid rust_device handle!" << std::endl;
        return nullptr;
    }
    
    RustTrackedDeviceServerDriver* wrapper = new RustTrackedDeviceServerDriver(rust_device);
    std::cout << "create_rust_device_wrapper: Created wrapper at " << wrapper << std::endl;
    return wrapper;
}
