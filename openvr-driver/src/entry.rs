//! Entry point module for OpenVR driver factory
//!
//! This module provides the internal implementation of the HmdDriverFactory
//! function that OpenVR calls to instantiate drivers.

use crate::{sys, ServerTrackedDeviceProvider};
use std::ffi::{c_char, c_int, c_void, CStr};
use std::sync::{Arc, Mutex, Once};

/// One-time initialization flag
static INIT: Once = Once::new();

/// Type-erased storage for the provider vtable
static mut PROVIDER_VTABLE: Option<*mut c_void> = None;

/// Create the entry point for a driver provider
///
/// This function is called by the `openvr_driver_entry!` macro to create
/// the actual HmdDriverFactory implementation.
///
/// # Safety
/// This function is unsafe because it:
/// - Dereferences raw pointers from OpenVR
/// - Uses static mutable storage for the provider instance
/// - Creates vtables that must match C++ ABI exactly
pub unsafe fn create_entry_point<T>(
    interface_name: *const c_char,
    return_code: *mut c_int,
) -> *mut c_void
where
    T: ServerTrackedDeviceProvider + Default + 'static,
{
    // Validate parameters
    if interface_name.is_null() || return_code.is_null() {
        return std::ptr::null_mut();
    }

    // Get the interface name
    let interface = match CStr::from_ptr(interface_name).to_str() {
        Ok(s) => s,
        Err(_) => {
            *return_code = sys::root::vr::EVRInitError::Init_InvalidInterface as i32;
            return std::ptr::null_mut();
        }
    };

    eprintln!(
        "[openvr-driver] HmdDriverFactory called for interface: {}",
        interface
    );

    // Check if requesting IServerTrackedDeviceProvider
    if interface.starts_with("IServerTrackedDeviceProvider") {
        // Initialize provider if not already done
        INIT.call_once(|| {
            let provider = T::default();
            let provider_arc = Arc::new(Mutex::new(provider));
            let vtable_ptr = crate::vtables::create_provider_vtable(provider_arc);
            PROVIDER_VTABLE = Some(vtable_ptr);
            eprintln!("[openvr-driver] Created provider instance and vtable");
        });

        if let Some(vtable_ptr) = PROVIDER_VTABLE {
            *return_code = 0; // Success

            eprintln!(
                "[openvr-driver] Returning IServerTrackedDeviceProvider vtable at {:?}",
                vtable_ptr
            );
            return vtable_ptr;
        }
    }

    eprintln!(
        "[openvr-driver] Interface {} not found or not supported",
        interface
    );
    *return_code = sys::root::vr::EVRInitError::Init_InterfaceNotFound as i32;
    std::ptr::null_mut()
}

/// Reset the provider instance
///
/// This is mainly useful for testing or when reloading the driver.
/// In normal operation, the provider instance persists for the lifetime
/// of the driver process.
///
/// # Safety
/// This function is unsafe because it modifies global mutable state.
/// It should only be called when no other threads are accessing the provider.
pub unsafe fn reset_provider() {
    PROVIDER_VTABLE = None;
}
