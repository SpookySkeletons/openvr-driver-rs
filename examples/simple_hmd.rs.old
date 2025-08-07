//! Simple HMD Driver Example
//!
//! A minimal OpenVR HMD driver that demonstrates the basic structure
//! and lifecycle of an OpenVR driver built with Rust.

use openvr_driver_rs::*;
use openvr_driver_sys as sys;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::ptr;
use std::time::{Duration, Instant};

// ============================================================================
// DEVICE PROVIDER
// ============================================================================

/// Main provider that manages the HMD device lifecycle
pub struct SimpleHmdProvider {
    hmd_driver: Option<SimpleHmdDriver>,
    driver_host: Option<*mut sys::vr::IVRServerDriverHost>,
}

impl SimpleHmdProvider {
    pub fn new() -> Self {
        Self {
            hmd_driver: None,
            driver_host: None,
        }
    }
}

impl ServerTrackedDeviceProvider for SimpleHmdProvider {
    fn init(&mut self, driver_context: &dyn DriverContext) -> Result<(), sys::vr::EVRInitError> {
        logging::info("SimpleHmdProvider: Initializing driver...");

        // Get the driver host interface for communicating with SteamVR
        if let Some(host_ptr) = driver_context.get_generic_interface("IVRServerDriverHost_006") {
            self.driver_host = Some(host_ptr as *mut sys::vr::IVRServerDriverHost);
            logging::info("SimpleHmdProvider: Connected to SteamVR host interface");
        } else {
            logging::error("SimpleHmdProvider: Failed to connect to SteamVR host interface");
            return Err(sys::vr::EVRInitError::VRInitError_Init_InterfaceNotFound);
        }

        // Create our HMD device
        let hmd = SimpleHmdDriver::new();

        // Register the device with SteamVR through the bridge system
        if let Some(host) = self.driver_host {
            self.register_device(&hmd, host)?;
        }

        self.hmd_driver = Some(hmd);
        logging::info("SimpleHmdProvider: Initialization complete");
        Ok(())
    }

    fn cleanup(&mut self) {
        logging::info("SimpleHmdProvider: Cleaning up...");
        self.hmd_driver = None;
        self.driver_host = None;
    }

    fn run_frame(&mut self) {
        if let Some(ref mut hmd) = self.hmd_driver {
            hmd.run_frame();
        }
    }

    fn should_block_standby_mode(&self) -> bool {
        false
    }

    fn enter_standby(&mut self) {
        logging::info("SimpleHmdProvider: Entering standby mode");
    }

    fn leave_standby(&mut self) {
        logging::info("SimpleHmdProvider: Leaving standby mode");
    }
}

impl SimpleHmdProvider {
    /// Register any device with SteamVR using the generic bridge
    fn register_device(
        &self,
        device: &SimpleHmdDriver,
        host: *mut sys::vr::IVRServerDriverHost,
    ) -> Result<(), sys::vr::EVRInitError> {
        let serial = CString::new(device.get_serial_number()).unwrap();

        logging::info(&format!(
            "SimpleHmdProvider: Registering device with serial: {}",
            device.get_serial_number()
        ));

        // Create a new device instance for the bridge (since we can't move the borrowed one)
        let bridge_device = Box::new(SimpleHmdDriver::new());

        // Create bridge wrapper for our device
        let rust_device_bridge = sys::bridge::create_device_bridge(bridge_device);

        if rust_device_bridge.is_null() {
            logging::error("SimpleHmdProvider: Failed to create device bridge");
            return Err(sys::vr::EVRInitError::VRInitError_Init_InitCanceledByUser);
        }

        // Create C++ device wrapper
        let cpp_device = unsafe { sys::bridge::create_cpp_device_wrapper(rust_device_bridge) };

        if cpp_device.is_null() {
            logging::error("SimpleHmdProvider: Failed to create C++ device wrapper");
            return Err(sys::vr::EVRInitError::VRInitError_Init_InitCanceledByUser);
        }

        // Register device with OpenVR
        let success = unsafe {
            sys::bridge::register_device_with_openvr(
                host as *mut std::ffi::c_void,
                serial.as_ptr(),
                device.get_device_class(),
                cpp_device,
            )
        };

        if success {
            logging::info("SimpleHmdProvider: Successfully registered device");
            Ok(())
        } else {
            logging::error("SimpleHmdProvider: Failed to register device");
            Err(sys::vr::EVRInitError::VRInitError_Init_InitCanceledByUser)
        }
    }
}

// ============================================================================
// HMD DEVICE DRIVER
// ============================================================================

/// Simple HMD device implementation
pub struct SimpleHmdDriver {
    device_id: Option<sys::vr::TrackedDeviceIndex_t>,
    last_pose_update: Instant,
}

impl SimpleHmdDriver {
    pub fn new() -> Self {
        Self {
            device_id: None,
            last_pose_update: Instant::now(),
        }
    }
}

impl TrackedDeviceServerDriver for SimpleHmdDriver {
    fn activate(
        &mut self,
        device_id: sys::vr::TrackedDeviceIndex_t,
    ) -> Result<(), sys::vr::EVRInitError> {
        self.device_id = Some(device_id);
        logging::info(&format!(
            "SimpleHMD: Device activated with ID {}",
            device_id
        ));
        Ok(())
    }

    fn deactivate(&mut self) {
        if let Some(device_id) = self.device_id {
            logging::info(&format!("SimpleHMD: Device {} deactivated", device_id));
        }
        self.device_id = None;
    }

    fn run_frame(&mut self) {
        // Update pose at standard VR refresh rate (~90 FPS)
        if self.last_pose_update.elapsed() > Duration::from_millis(90 / 1000) {
            // In a real driver, you would:
            // 1. Get tracking data from your hardware
            // 2. Convert to OpenVR pose format
            // 3. Submit pose update to SteamVR
            self.last_pose_update = Instant::now();
        }
    }

    fn get_serial_number(&self) -> String {
        "SIMPLE_HMD_001".to_string()
    }

    fn enter_standby(&mut self) {
        logging::info("SimpleHMD: Entering standby mode");
    }

    fn get_device_class(&self) -> sys::vr::ETrackedDeviceClass {
        sys::vr::ETrackedDeviceClass::TrackedDeviceClass_HMD
    }
}

// ============================================================================
// DRIVER FACTORY
// ============================================================================

/// Register the provider factory with the bridge system
fn register_provider_factory() {
    sys::bridge::register_provider_factory(|| {
        logging::info("Creating SimpleHmdProvider instance");
        Box::new(SimpleHmdProvider::new())
    });
}

/// Main entry point - SteamVR calls this function to load the driver
#[unsafe(no_mangle)]
pub unsafe extern "C" fn HmdDriverFactory(
    interface_name: *const c_char,
    return_code: *mut sys::vr::EVRInitError,
) -> *mut c_void {
    // Validate input parameters
    if interface_name.is_null() {
        *return_code = sys::vr::EVRInitError::VRInitError_Init_InvalidInterface;
        return ptr::null_mut();
    }

    // Convert interface name to Rust string
    let interface_name = unsafe { CStr::from_ptr(interface_name) };
    let interface_str = match interface_name.to_str() {
        Ok(s) => s,
        Err(_) => {
            *return_code = sys::vr::EVRInitError::VRInitError_Init_InvalidInterface;
            return ptr::null_mut();
        }
    };

    logging::info(&format!(
        "HmdDriverFactory: SteamVR requested interface '{}'",
        interface_str
    ));

    // Handle the interface request
    match interface_str {
        "IServerTrackedDeviceProvider_004" => {
            logging::info("HmdDriverFactory: Creating device provider");

            // Register our provider factory
            register_provider_factory();

            *return_code = sys::vr::EVRInitError::VRInitError_None;
            sys::bridge::create_provider_wrapper()
        }
        _ => {
            logging::warn(&format!(
                "HmdDriverFactory: Unsupported interface '{}'",
                interface_str
            ));
            unsafe { *return_code = sys::vr::EVRInitError::VRInitError_Init_InvalidInterface };
            ptr::null_mut()
        }
    }
}
