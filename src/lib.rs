//! # OpenVR Driver Rust Bindings
//!
//! Pure Rust bindings for creating OpenVR drivers. This crate provides
//! a safe, idiomatic Rust interface for implementing OpenVR device drivers
//! without any C++ dependencies.
//!
//! ## Usage
//!
//! Implement the required traits to create your driver:
//!
//! ```rust,no_run
//! use openvr_driver_rs::*;
//!
//! struct MyProvider;
//! impl ServerTrackedDeviceProvider for MyProvider {
//!     // ... implement required methods
//! }
//!
//! struct MyDevice;
//! impl TrackedDeviceServerDriver for MyDevice {
//!     // ... implement required methods
//! }
//! ```

// Re-export the bindings and macros
pub use driver_macros::InterfaceImpl;
pub use openvr_driver_bindings as sys;

// Re-export essential types
pub use sys::{
    DriverPose_t, ETrackedDeviceClass, EVRInitError, PropertyContainerHandle_t,
    TrackedDeviceIndex_t, VRControllerState_t,
};

// Re-export main interfaces
pub use sys::{
    IServerTrackedDeviceProvider, ITrackedDeviceServerDriver, IVRDriverContext, IVRServerDriverHost,
};

use std::ffi::c_void;

/// Trait for implementing an OpenVR server tracked device provider
pub trait ServerTrackedDeviceProvider: Send + Sync {
    /// Initialize the driver with the provided context
    fn init(&mut self, driver_context: *mut c_void) -> Result<(), EVRInitError>;

    /// Cleanup the driver
    fn cleanup(&mut self);

    /// Get the list of interface versions this driver supports
    fn get_interface_versions(&self) -> Vec<&'static str>;

    /// Called every frame
    fn run_frame(&mut self);

    /// Should return true if the driver wants to block standby mode
    fn should_block_standby_mode(&self) -> bool;

    /// Called when the system is entering standby
    fn enter_standby(&mut self);

    /// Called when the system is leaving standby
    fn leave_standby(&mut self);
}

/// Trait for implementing a tracked device
pub trait TrackedDeviceServerDriver: Send + Sync {
    /// Activate the device with the given index
    fn activate(&mut self, device_index: u32) -> Result<(), EVRInitError>;

    /// Deactivate the device
    fn deactivate(&mut self);

    /// Called when entering standby
    fn enter_standby(&mut self);

    /// Get a component interface (e.g., display, controller)
    fn get_component(&self, component_name: &str) -> Option<*mut c_void>;

    /// Handle debug requests
    fn debug_request(&mut self, request: &str) -> String;

    /// Get the current pose of the device
    fn get_pose(&self) -> DriverPose_t;
}

/// Result type for driver operations
pub type DriverResult<T> = Result<T, EVRInitError>;

/// Helper macro for implementing the HmdDriverFactory entry point
#[macro_export]
macro_rules! openvr_driver_entry {
    ($provider_type:ty) => {
        #[no_mangle]
        pub unsafe extern "C" fn HmdDriverFactory(
            interface_name: *const std::ffi::c_char,
            return_code: *mut i32,
        ) -> *mut std::ffi::c_void {
            use std::ffi::CStr;

            if interface_name.is_null() || return_code.is_null() {
                return std::ptr::null_mut();
            }

            let interface = CStr::from_ptr(interface_name);

            // Check if requesting IServerTrackedDeviceProvider
            if interface
                .to_bytes()
                .starts_with(b"IServerTrackedDeviceProvider")
            {
                *return_code = 0; // Success
                                  // TODO: Create and return vtable for provider
                                  // For now return null until vtable system is complete
                return std::ptr::null_mut();
            }

            *return_code = 108; // VRInitError_Init_InterfaceNotFound
            std::ptr::null_mut()
        }
    };
}
