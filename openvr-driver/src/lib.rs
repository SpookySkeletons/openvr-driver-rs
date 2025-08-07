//! High-level safe Rust bindings for OpenVR driver development
//!
//! This crate provides a safe, idiomatic Rust interface for developing
//! OpenVR drivers without dealing with low-level FFI details.
//!
//! # Example
//!
//! ```no_run
//! use openvr_driver::prelude::*;
//!
//! struct MyDriver {
//!     // Your driver state
//! }
//!
//! impl ServerTrackedDeviceProvider for MyDriver {
//!     fn init(&mut self, context: &mut DriverContext) -> DriverResult<()> {
//!         // Initialize your driver
//!         Ok(())
//!     }
//!
//!     fn cleanup(&mut self) {
//!         // Cleanup resources
//!     }
//!
//!     fn run_frame(&mut self) {
//!         // Called every frame
//!     }
//!
//!     fn should_block_standby_mode(&self) -> bool {
//!         false
//!     }
//!
//!     fn enter_standby(&mut self) {}
//!     fn leave_standby(&mut self) {}
//! }
//!
//! openvr_driver_entry!(MyDriver);
//! ```

#![allow(non_snake_case)]

// Re-export the sys crate for advanced users who need raw access
pub use openvr_driver_bindings as sys;

// Re-export macros
pub use driver_macros as macros;

// Core modules
pub mod context;
mod entry;
pub mod error;
pub mod interfaces;
pub mod properties;
mod vtables;

// Public API exports
pub use context::{DriverContext, DriverHost};
pub use entry::create_entry_point;
pub use error::{DriverError, DriverResult};
pub use properties::{Property, PropertyContainer, PropertyValue, PropertyWrite};

// Interface traits that users implement
pub use interfaces::{
    CameraComponent, Component, ComponentResult, ControllerComponent, DisplayComponent,
    DriverInput, Eye, ServerTrackedDeviceProvider, TrackedDeviceServerDriver, VirtualDisplay,
    WatchdogProvider,
};

// Configuration types
pub use interfaces::{
    ActivationState, CameraConfiguration, ControllerConfiguration, ControllerRole,
    DeviceProperties, DisplayConfiguration,
};

// Helper types from OpenVR
pub use sys::root::vr::{
    DriverPose_t as DriverPose, ETrackedDeviceClass as DeviceClass,
    ETrackedPropertyError as PropertyError, EVRInitError as InitError,
    HmdMatrix34_t as HmdMatrix34, HmdQuaternion_t as HmdQuaternion, HmdVector3_t as HmdVector3,
    HmdVector3d_t as HmdVector3d, PropertyContainerHandle_t as PropertyHandle,
    TrackedDeviceIndex_t as TrackedDeviceIndex, VRControllerState_t as ControllerState,
    VRInputComponentHandle_t as InputComponentHandle,
};

// Re-export display component types
pub use sys::root::vr::{
    EVRDistortionFunctionType as DistortionFunctionType,
    EVRMuraCorrectionMode as MuraCorrectionMode,
};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::{
        openvr_driver_entry, ComponentResult, DeviceClass, DisplayComponent, DisplayConfiguration,
        DriverContext, DriverError, DriverHost, DriverPose, DriverResult, Eye, InitError, Property,
        PropertyValue, ServerTrackedDeviceProvider, TrackedDeviceServerDriver,
    };
}

/// Macro to generate the driver entry point
///
/// This macro creates the `HmdDriverFactory` function that OpenVR calls
/// to instantiate your driver.
///
/// # Example
/// ```no_run
/// use openvr_driver::prelude::*;
///
/// struct MyDriver;
///
/// impl ServerTrackedDeviceProvider for MyDriver {
///     // ... implementation
/// }
///
/// openvr_driver_entry!(MyDriver);
/// ```
#[macro_export]
macro_rules! openvr_driver_entry {
    ($provider:ty) => {
        #[no_mangle]
        pub unsafe extern "C" fn HmdDriverFactory(
            interface_name: *const ::std::os::raw::c_char,
            return_code: *mut ::std::os::raw::c_int,
        ) -> *mut ::std::os::raw::c_void {
            $crate::create_entry_point::<$provider>(interface_name, return_code)
        }

        // Additional exports that some OpenVR versions expect
        #[no_mangle]
        pub extern "C" fn HmdDriverFactory_GetInterfaceVersions(
        ) -> *const *const ::std::os::raw::c_char {
            // Use a wrapper struct to make the array Sync
            struct InterfaceVersions {
                versions: [*const ::std::os::raw::c_char; 3],
            }

            unsafe impl Sync for InterfaceVersions {}

            static PROVIDER_VERSION: &[u8] = b"IServerTrackedDeviceProvider_005\0";
            static DEVICE_VERSION: &[u8] = b"ITrackedDeviceServerDriver_005\0";

            static VERSIONS: InterfaceVersions = InterfaceVersions {
                versions: [
                    PROVIDER_VERSION.as_ptr() as *const ::std::os::raw::c_char,
                    DEVICE_VERSION.as_ptr() as *const ::std::os::raw::c_char,
                    ::std::ptr::null(),
                ],
            };

            VERSIONS.versions.as_ptr()
        }
    };
}

/// Settings helper utilities
pub mod settings {
    use crate::sys;
    use std::ffi::{CStr, CString};

    /// Read a string setting from OpenVR
    pub fn get_string(section: &str, key: &str, default: &str) -> String {
        // TODO: Settings need to be accessed through the driver context
        // For now, return the default value
        eprintln!(
            "[Settings] Would read string setting {}.{} (default: {})",
            section, key, default
        );
        default.to_string()
    }

    /// Read a float setting from OpenVR
    pub fn get_float(section: &str, key: &str, default: f32) -> f32 {
        // TODO: Settings need to be accessed through the driver context
        // For now, return the default value
        eprintln!(
            "[Settings] Would read float setting {}.{} (default: {})",
            section, key, default
        );
        default
    }

    /// Read an integer setting from OpenVR
    pub fn get_int32(section: &str, key: &str, default: i32) -> i32 {
        // TODO: Settings need to be accessed through the driver context
        // For now, return the default value
        eprintln!(
            "[Settings] Would read int32 setting {}.{} (default: {})",
            section, key, default
        );
        default
    }

    /// Read a boolean setting from OpenVR
    pub fn get_bool(section: &str, key: &str, default: bool) -> bool {
        // TODO: Settings need to be accessed through the driver context
        // For now, return the default value
        eprintln!(
            "[Settings] Would read bool setting {}.{} (default: {})",
            section, key, default
        );
        default
    }
}

/// Check if there's a pending device activation from OpenVR and take it
///
/// This is used by driver implementations to check if OpenVR has called
/// the activate callback on a device. Due to Arc<dyn Trait> limitations,
/// the vtable stores the activation index globally for retrieval.
pub fn take_pending_device_activation() -> Option<u32> {
    vtables::device::take_pending_activation()
}
