//! Bridge system for connecting Rust driver implementations to OpenVR C++ interfaces
//!
//! This module provides the bridge between OpenVR's C++ virtual interface system
//! and Rust trait implementations. It handles the conversion and forwarding of
//! calls between the two systems.
//!
//! ## Generic Bridge API
//!
//! This bridge is designed to be completely generic - it works with any implementation
//! of the core traits without knowing about specific device types.
//!
//! ### For Driver Developers
//!
//! ```rust
//! use openvr_driver_rs::bridge;
//!
//! // 1. Create your device that implements TrackedDeviceServerDriver
//! let my_device = Box::new(MyDevice::new());
//!
//! // 2. Wrap it in the bridge system
//! let bridge_ptr = bridge::create_device_bridge(my_device);
//!
//! // 3. Create C++ wrapper for OpenVR
//! let cpp_device = unsafe { bridge::create_cpp_device_wrapper(bridge_ptr) };
//!
//! // 4. Register with OpenVR
//! let success = unsafe {
//!     bridge::register_device_with_openvr(
//!         host_ptr,
//!         serial_cstring.as_ptr(),
//!         device_class,
//!         cpp_device,
//!     )
//! };
//! ```
//!
//! The bridge handles all the complexity of converting between Rust traits and
//! OpenVR C++ virtual interfaces.

use crate::ffi::vr;
use crate::traits::{DriverContext, ServerTrackedDeviceProvider, TrackedDeviceServerDriver};
use std::boxed::Box;
use std::ffi::c_void;

// ============================================================================
// DRIVER CONTEXT BRIDGE
// ============================================================================

/// Bridge implementation of DriverContext that forwards to C++ OpenVR context
struct BridgeDriverContext {
    raw_context: *mut c_void,
}

impl BridgeDriverContext {
    fn new(raw_context: *mut c_void) -> Self {
        Self { raw_context }
    }
}

impl DriverContext for BridgeDriverContext {
    fn get_generic_interface(&self, interface_version: &str) -> Option<*mut std::ffi::c_void> {
        if self.raw_context.is_null() {
            return None;
        }

        unsafe {
            let interface_cstr = std::ffi::CString::new(interface_version).ok()?;
            let mut error = 0i32;

            let result = driver_context_get_generic_interface(
                self.raw_context,
                interface_cstr.as_ptr(),
                &mut error,
            );

            if error == 0 && !result.is_null() {
                Some(result)
            } else {
                None
            }
        }
    }

    fn get_driver_handle(&self) -> vr::DriverHandle_t {
        if self.raw_context.is_null() {
            return 0;
        }
        unsafe { driver_context_get_driver_handle(self.raw_context) }
    }
}

// ============================================================================
// PROVIDER BRIDGE
// ============================================================================

/// Global provider factory function - set by the driver implementation
static mut PROVIDER_FACTORY: Option<fn() -> Box<dyn ServerTrackedDeviceProvider>> = None;

/// Register the provider factory function
pub fn register_provider_factory(factory: fn() -> Box<dyn ServerTrackedDeviceProvider>) {
    unsafe {
        PROVIDER_FACTORY = Some(factory);
    }
}

/// Bridge wrapper for ServerTrackedDeviceProvider
struct BridgeProvider {
    inner: Box<dyn ServerTrackedDeviceProvider>,
}

impl BridgeProvider {
    fn new(provider: Box<dyn ServerTrackedDeviceProvider>) -> Self {
        Self { inner: provider }
    }

    fn init(&mut self, driver_context: *mut c_void) -> bool {
        let context = BridgeDriverContext::new(driver_context);
        self.inner.init(&context).is_ok()
    }

    fn cleanup(&mut self) {
        self.inner.cleanup();
    }

    fn run_frame(&mut self) {
        self.inner.run_frame();
    }

    fn should_block_standby_mode(&self) -> bool {
        self.inner.should_block_standby_mode()
    }

    fn enter_standby(&mut self) {
        self.inner.enter_standby();
    }

    fn leave_standby(&mut self) {
        self.inner.leave_standby();
    }
}

// ============================================================================
// DEVICE BRIDGE
// ============================================================================

/// Bridge wrapper for TrackedDeviceServerDriver
struct BridgeDevice {
    inner: Box<dyn TrackedDeviceServerDriver>,
}

impl BridgeDevice {
    fn new(device: Box<dyn TrackedDeviceServerDriver>) -> Self {
        Self { inner: device }
    }

    fn activate(&mut self, device_id: u32) -> bool {
        self.inner.activate(device_id).is_ok()
    }

    fn deactivate(&mut self) {
        self.inner.deactivate();
    }

    fn run_frame(&mut self) {
        self.inner.run_frame();
    }

    fn enter_standby(&mut self) {
        self.inner.enter_standby();
    }

    fn get_serial_number(&self) -> String {
        self.inner.get_serial_number()
    }

    fn get_device_class(&self) -> vr::ETrackedDeviceClass {
        self.inner.get_device_class()
    }
}

// ============================================================================
// C INTERFACE - PROVIDER FUNCTIONS
// ============================================================================

/// Create a new provider instance
#[unsafe(no_mangle)]
pub extern "C" fn rust_provider_create() -> *mut c_void {
    let provider = unsafe {
        if let Some(factory) = PROVIDER_FACTORY {
            factory()
        } else {
            eprintln!("FATAL ERROR: No provider factory registered!");
            eprintln!(
                "This means your driver called create_provider_wrapper() without first calling register_provider_factory()."
            );
            eprintln!("This is a programming error in your driver implementation.");
            eprintln!("The driver will now fail to load - fix your driver code!");
            return std::ptr::null_mut();
        }
    };

    let bridge_provider = Box::new(BridgeProvider::new(provider));
    Box::into_raw(bridge_provider) as *mut c_void
}

/// Destroy a provider instance
#[unsafe(no_mangle)]
pub extern "C" fn rust_provider_destroy(handle: *mut c_void) {
    if !handle.is_null() {
        unsafe {
            let _provider = Box::from_raw(handle as *mut BridgeProvider);
        }
    }
}

/// Initialize the provider
#[unsafe(no_mangle)]
pub extern "C" fn rust_provider_init(handle: *mut c_void, driver_context: *mut c_void) -> i32 {
    if handle.is_null() {
        return -1;
    }

    unsafe {
        let provider = &mut *(handle as *mut BridgeProvider);
        if provider.init(driver_context) { 0 } else { -1 }
    }
}

/// Clean up the provider
#[unsafe(no_mangle)]
pub extern "C" fn rust_provider_cleanup(handle: *mut c_void) {
    if !handle.is_null() {
        unsafe {
            let provider = &mut *(handle as *mut BridgeProvider);
            provider.cleanup();
        }
    }
}

/// Run per-frame updates
#[unsafe(no_mangle)]
pub extern "C" fn rust_provider_run_frame(handle: *mut c_void) {
    if !handle.is_null() {
        unsafe {
            let provider = &mut *(handle as *mut BridgeProvider);
            provider.run_frame();
        }
    }
}

/// Check if provider should block standby mode
#[unsafe(no_mangle)]
pub extern "C" fn rust_provider_should_block_standby(handle: *mut c_void) -> i32 {
    if !handle.is_null() {
        unsafe {
            let provider = &*(handle as *mut BridgeProvider);
            if provider.should_block_standby_mode() {
                1
            } else {
                0
            }
        }
    } else {
        0
    }
}

/// Enter standby mode
#[unsafe(no_mangle)]
pub extern "C" fn rust_provider_enter_standby(handle: *mut c_void) {
    if !handle.is_null() {
        unsafe {
            let provider = &mut *(handle as *mut BridgeProvider);
            provider.enter_standby();
        }
    }
}

/// Leave standby mode
#[unsafe(no_mangle)]
pub extern "C" fn rust_provider_leave_standby(handle: *mut c_void) {
    if !handle.is_null() {
        unsafe {
            let provider = &mut *(handle as *mut BridgeProvider);
            provider.leave_standby();
        }
    }
}

// ============================================================================
// C INTERFACE - DEVICE FUNCTIONS
// ============================================================================

/// Create a bridge wrapper for any TrackedDeviceServerDriver
/// This is the main Rust API - drivers call this directly with their device implementation
pub fn create_device_bridge(device: Box<dyn TrackedDeviceServerDriver>) -> *mut c_void {
    let bridge_device = Box::new(BridgeDevice::new(device));
    Box::into_raw(bridge_device) as *mut c_void
}

/// Destroy a device instance
#[unsafe(no_mangle)]
pub extern "C" fn rust_device_destroy(handle: *mut c_void) {
    if !handle.is_null() {
        unsafe {
            let _device = Box::from_raw(handle as *mut BridgeDevice);
        }
    }
}

/// Activate a device
#[unsafe(no_mangle)]
pub extern "C" fn rust_device_activate(handle: *mut c_void, device_id: u32) -> i32 {
    if handle.is_null() {
        return -1;
    }

    unsafe {
        let device = &mut *(handle as *mut BridgeDevice);
        if device.activate(device_id) { 0 } else { -1 }
    }
}

/// Deactivate a device
#[unsafe(no_mangle)]
pub extern "C" fn rust_device_deactivate(handle: *mut c_void) {
    if !handle.is_null() {
        unsafe {
            let device = &mut *(handle as *mut BridgeDevice);
            device.deactivate();
        }
    }
}

/// Run per-frame device updates
#[unsafe(no_mangle)]
pub extern "C" fn rust_device_run_frame(handle: *mut c_void) {
    if !handle.is_null() {
        unsafe {
            let device = &mut *(handle as *mut BridgeDevice);
            device.run_frame();
        }
    }
}

/// Enter device standby mode
#[unsafe(no_mangle)]
pub extern "C" fn rust_device_enter_standby(handle: *mut c_void) {
    if !handle.is_null() {
        unsafe {
            let device = &mut *(handle as *mut BridgeDevice);
            device.enter_standby();
        }
    }
}

// ============================================================================
// C INTERFACE DECLARATIONS
// ============================================================================

unsafe extern "C" {
    fn driver_context_get_generic_interface(
        context: *mut c_void,
        interface_version: *const std::ffi::c_char,
        error: *mut i32,
    ) -> *mut c_void;
    
    fn driver_context_get_driver_handle(context: *mut c_void) -> u64;
    
    fn server_driver_host_tracked_device_added(
        host_ptr: *mut c_void,
        serial_number: *const std::ffi::c_char,
        device_class: i32,
        device_driver_ptr: *mut c_void,
    ) -> bool;
    
    fn create_rust_device_wrapper(rust_device_bridge: *mut c_void) -> *mut c_void;
    fn create_rust_server_provider() -> *mut c_void;
}

// ============================================================================
// PUBLIC API
// ============================================================================

/// Create the provider wrapper (called by driver factory)
pub fn create_provider_wrapper() -> *mut c_void {
    unsafe { create_rust_server_provider() }
}

/// Register a device with the OpenVR system
/// This is a direct wrapper around the C++ OpenVR interface
pub unsafe fn register_device_with_openvr(
    host_ptr: *mut c_void,
    serial_number: *const std::ffi::c_char,
    device_class: vr::ETrackedDeviceClass,
    device_driver_ptr: *mut c_void,
) -> bool {
    server_driver_host_tracked_device_added(
        host_ptr,
        serial_number,
        device_class as i32,
        device_driver_ptr,
    )
}

/// Create a C++ wrapper for a Rust device bridge
/// This prepares the device for registration with OpenVR
pub unsafe fn create_cpp_device_wrapper(rust_device_bridge: *mut c_void) -> *mut c_void {
    create_rust_device_wrapper(rust_device_bridge)
}
