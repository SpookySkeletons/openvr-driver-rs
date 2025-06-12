//! Complete OpenVR HMD driver example
//! This builds as a .so/.dll that SteamVR can load

use openvr_driver_rs::*;
use openvr_driver_sys as sys;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::ptr;
use std::time::{Duration, Instant};

// ===== DRIVER IMPLEMENTATION =====

/// Complete driver provider that manages our HMD
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
        println!("SimpleHmdProvider: Initializing...");

        // Get the driver host interface (this is how we talk back to SteamVR)
        if let Some(host_ptr) = driver_context.get_generic_interface("IVRServerDriverHost_006") {
            self.driver_host = Some(host_ptr as *mut sys::vr::IVRServerDriverHost);
            println!("SimpleHmdProvider: Got driver host interface");
        } else {
            println!("SimpleHmdProvider: Failed to get driver host interface");
            return Err(sys::vr::EVRInitError::VRInitError_Init_InterfaceNotFound);
        }

        // Create and register our HMD device
        let mut hmd = SimpleHmdDriver::new();

        // Tell SteamVR about our device
        if let Some(host) = self.driver_host {
            let serial = CString::new(hmd.get_serial_number()).unwrap();
            // This is where we'd call TrackedDeviceAdded - for now just print
            println!(
                "SimpleHmdProvider: Would register HMD with serial: {}",
                hmd.get_serial_number()
            );
            // let success = (*host).TrackedDeviceAdded(serial.as_ptr(), hmd.get_device_class(), &mut hmd as *mut _ as *mut _);
        }

        self.hmd_driver = Some(hmd);
        println!("SimpleHmdProvider: Initialization complete");
        Ok(())
    }

    fn cleanup(&mut self) {
        println!("SimpleHmdProvider: Cleaning up...");
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
    fn enter_standby(&mut self) {}
    fn leave_standby(&mut self) {}
}

/// Simple HMD driver implementation
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
        println!("SimpleHMD: Activated with device ID {}", device_id);
        Ok(())
    }

    fn deactivate(&mut self) {
        println!("SimpleHMD: Deactivated");
        self.device_id = None;
    }

    fn run_frame(&mut self) {
        // Update pose at ~90 FPS
        if self.last_pose_update.elapsed() > Duration::from_millis(11) {
            // Here we'd send pose updates to SteamVR
            self.last_pose_update = Instant::now();
        }
    }

    fn get_serial_number(&self) -> String {
        "SIMPLE_HMD_001".to_string()
    }

    fn enter_standby(&mut self) {}

    fn get_device_class(&self) -> sys::vr::ETrackedDeviceClass {
        sys::vr::ETrackedDeviceClass::TrackedDeviceClass_HMD
    }
}

// ===== C INTERFACE FOR STEAMVR =====

/// Register our provider factory with the bridge
fn register_simple_hmd_provider() {
    sys::bridge::register_provider_factory(|| {
        println!("SimpleHmdProvider factory: Creating SimpleHmdProvider");
        Box::new(SimpleHmdProvider::new())
    });
}

/// Main entry point - SteamVR calls this to get your driver
#[unsafe(no_mangle)]
pub extern "C" fn HmdDriverFactory(
    interface_name: *const c_char,
    return_code: *mut sys::vr::EVRInitError,
) -> *mut c_void {
    if interface_name.is_null() {
        unsafe { *return_code = sys::vr::EVRInitError::VRInitError_Init_InvalidInterface };
        return ptr::null_mut();
    }

    let interface_name = unsafe { CStr::from_ptr(interface_name) };
    let interface_str = interface_name.to_str().unwrap_or("");

    println!("HmdDriverFactory: Requested interface '{}'", interface_str);

    match interface_str {
        "IServerTrackedDeviceProvider_004" => {
            println!("HmdDriverFactory: Registering SimpleHmdProvider and creating C++ wrapper!");
            // Register our provider factory first
            register_simple_hmd_provider();
            
            unsafe {
                *return_code = sys::vr::EVRInitError::VRInitError_None;
                // Use the bridge to create the C++ wrapper
                sys::bridge::create_provider_wrapper()
            }
        }
        _ => {
            println!(
                "HmdDriverFactory: Interface '{}' not supported",
                interface_str
            );
            unsafe { *return_code = sys::vr::EVRInitError::VRInitError_Init_InvalidInterface };
            ptr::null_mut()
        }
    }
}
