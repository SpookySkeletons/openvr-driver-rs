#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]
#![allow(clippy::all)]

// Include the generated bindings
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::ffi::{c_char, c_void};

// Re-export key types that drivers will need
pub use root::vr::DriverPose_t;
pub use root::vr::ETrackedDeviceClass;
pub use root::vr::EVRInitError;
pub use root::vr::PropertyContainerHandle_t;
pub use root::vr::TrackedDeviceIndex_t;
pub use root::vr::VRControllerState_t;

// Export the main interfaces
pub use root::vr::IServerTrackedDeviceProvider;
pub use root::vr::ITrackedDeviceServerDriver;
pub use root::vr::IVRDriverContext;
pub use root::vr::IVRServerDriverHost;

// Helper trait for converting Rust implementations to C++ vtables
pub trait AsVtablePtr {
    fn as_vtable_ptr(&self) -> *mut c_void;
}

// Entry point that OpenVR will call
#[no_mangle]
pub unsafe extern "C" fn HmdDriverFactory(
    interface_name: *const c_char,
    return_code: *mut i32,
) -> *mut c_void {
    // This will be implemented by the driver crate
    // For now, return null with a generic error
    if !return_code.is_null() {
        *return_code = 108; // VRInitError_Init_InterfaceNotFound
    }
    std::ptr::null_mut()
}

// Module for version-specific interface implementations
pub mod interfaces {
    use super::*;

    // This is where we'll generate version-specific traits
    // For now, just a placeholder
    pub trait IServerTrackedDeviceProvider_Interface {
        fn init(&self, driver_context: *mut c_void) -> EVRInitError;
        fn cleanup(&self);
        fn get_interface_versions(&self) -> *const *const c_char;
        fn run_frame(&self);
        fn should_block_standby_mode(&self) -> bool;
        fn enter_standby(&self);
        fn leave_standby(&self);
    }

    pub trait ITrackedDeviceServerDriver_Interface {
        fn activate(&self, device_index: u32) -> EVRInitError;
        fn deactivate(&self);
        fn enter_standby(&self);
        fn get_component(&self, component_name: *const c_char) -> *mut c_void;
        fn debug_request(
            &self,
            request: *const c_char,
            response_buffer: *mut c_char,
            response_buffer_size: u32,
        );
        fn get_pose(&self) -> DriverPose_t;
    }
}

// Helper for creating vtable wrappers
#[macro_export]
macro_rules! impl_vtable_wrapper {
    ($interface:ident, $rust_impl:ty) => {
        impl $crate::AsVtablePtr for $rust_impl {
            fn as_vtable_ptr(&self) -> *mut std::ffi::c_void {
                // This will be expanded to create proper vtable
                std::ptr::null_mut()
            }
        }
    };
}
