//! Simple HMD Driver Example
//!
//! A minimal OpenVR HMD driver implementation in pure Rust that demonstrates
//! the basic structure and lifecycle of an OpenVR driver.

#![allow(dead_code)]

mod device;
mod provider;

use device::HmdDevice;
use provider::{ProviderWrapper, SimpleHmdProvider};

use openvr_driver_bindings::{create_provider_wrapper, root::vr::EVRInitError};
use std::ffi::{c_char, c_void, CStr};
use std::sync::{Arc, Mutex};

// Component types for input
#[derive(Debug, Clone, Copy)]
enum MyComponent {
    SystemTouch,
    SystemClick,
}

// Display configuration
#[derive(Debug, Clone)]
pub struct DisplayConfiguration {
    window_x: i32,
    window_y: i32,
    window_width: i32,
    window_height: i32,
    render_width: i32,
    render_height: i32,
}

impl Default for DisplayConfiguration {
    fn default() -> Self {
        Self {
            window_x: 0,
            window_y: 0,
            window_width: 1920,
            window_height: 1080,
            render_width: 1920,
            render_height: 1080,
        }
    }
}

// Global provider instance
use once_cell::sync::Lazy;
static PROVIDER: Lazy<Mutex<Option<Arc<Mutex<SimpleHmdProvider>>>>> =
    Lazy::new(|| Mutex::new(None));

// Entry point that OpenVR will call
#[no_mangle]
pub unsafe extern "C" fn HmdDriverFactory(
    interface_name: *const c_char,
    return_code: *mut i32,
) -> *mut c_void {
    if interface_name.is_null() || return_code.is_null() {
        return std::ptr::null_mut();
    }

    let interface = CStr::from_ptr(interface_name);
    let interface_str = interface.to_string_lossy();

    eprintln!(
        "SimpleHMD: HmdDriverFactory called for interface: {}",
        interface_str
    );

    // Check if requesting IServerTrackedDeviceProvider
    if interface_str.starts_with("IServerTrackedDeviceProvider") {
        // Initialize provider if not already done
        let mut provider_guard = PROVIDER.lock().unwrap();
        if provider_guard.is_none() {
            *provider_guard = Some(SimpleHmdProvider::new());
        }

        if let Some(ref provider) = *provider_guard {
            *return_code = 0; // Success

            // Create and return the vtable wrapper for the provider
            let vtable_ptr = create_provider_wrapper(ProviderWrapper(provider.clone()));

            eprintln!(
                "SimpleHMD: Returning IServerTrackedDeviceProvider vtable at {:?}",
                vtable_ptr
            );
            return vtable_ptr;
        }
    }

    eprintln!("SimpleHMD: Interface {} not found", interface_str);
    *return_code = EVRInitError::Init_InterfaceNotFound as i32;
    std::ptr::null_mut()
}

// Wrapper struct to make pointer array Sync
struct InterfaceVersions {
    versions: [*const c_char; 3],
}

unsafe impl Sync for InterfaceVersions {}

// Additional exports that some OpenVR versions expect
#[no_mangle]
pub extern "C" fn HmdDriverFactory_GetInterfaceVersions() -> *const *const c_char {
    // Return pointer to null-terminated array of interface version strings
    static PROVIDER_VERSION: &[u8] = b"IServerTrackedDeviceProvider_005\0";
    static DEVICE_VERSION: &[u8] = b"ITrackedDeviceServerDriver_005\0";

    static VERSIONS: InterfaceVersions = InterfaceVersions {
        versions: [
            PROVIDER_VERSION.as_ptr() as *const c_char,
            DEVICE_VERSION.as_ptr() as *const c_char,
            std::ptr::null(),
        ],
    };

    VERSIONS.versions.as_ptr()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_config_default() {
        let config = DisplayConfiguration::default();
        assert_eq!(config.window_width, 1920);
        assert_eq!(config.window_height, 1080);
    }

    #[test]
    fn test_provider_creation() {
        let provider = SimpleHmdProvider::new();
        assert!(provider.lock().is_ok());
    }

    #[test]
    fn test_hmd_creation() {
        let hmd = HmdDevice::new();
        assert_eq!(hmd.get_serial_number(), "SIMPLEHMD_001");
    }
}
