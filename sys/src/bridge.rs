use crate::ffi::vr;
use crate::traits::{DriverContext, ServerTrackedDeviceProvider};
use std::boxed::Box;
use std::ffi::c_void;

// Wrapper that implements DriverContext for the bridge
struct BridgeDriverContext {
    raw_context: *mut c_void,
}

impl BridgeDriverContext {
    fn new(raw_context: *mut c_void) -> Self {
        Self { raw_context }
    }
}

// Generic provider creation - no assumptions about which driver
static mut PROVIDER_FACTORY: Option<fn() -> Box<dyn ServerTrackedDeviceProvider>> = None;

// Function for examples to register their provider factory
pub fn register_provider_factory(factory: fn() -> Box<dyn ServerTrackedDeviceProvider>) {
    unsafe {
        PROVIDER_FACTORY = Some(factory);
    }
}

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
    fn create_rust_device_wrapper(rust_device: *mut c_void) -> *mut c_void;
}

impl DriverContext for BridgeDriverContext {
    fn get_generic_interface(&self, interface_version: &str) -> Option<*mut std::ffi::c_void> {
        println!(
            "BridgeDriverContext: get_generic_interface called for '{}'",
            interface_version
        );

        if self.raw_context.is_null() {
            println!("BridgeDriverContext: raw_context is null!");
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
                println!(
                    "BridgeDriverContext: Successfully got interface '{}'",
                    interface_version
                );
                Some(result)
            } else {
                println!(
                    "BridgeDriverContext: Failed to get interface '{}', error: {}",
                    interface_version, error
                );
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

// This is the actual provider that will use your trait system
// Later, this will be replaced by your SimpleHmdProvider from simple_hmd.rs
struct BridgeProvider {
    inner: Box<dyn ServerTrackedDeviceProvider>,
}

impl BridgeProvider {
    fn new(provider: Box<dyn ServerTrackedDeviceProvider>) -> Self {
        Self { inner: provider }
    }

    fn init(&mut self, driver_context: *mut c_void) -> bool {
        println!(
            "BridgeProvider: Initializing with driver context {:p}",
            driver_context
        );

        let context = BridgeDriverContext::new(driver_context);
        match self.inner.init(&context) {
            Ok(()) => {
                println!("BridgeProvider: Initialization successful!");
                true
            }
            Err(e) => {
                println!("BridgeProvider: Initialization failed!");
                false
            }
        }
    }

    fn cleanup(&mut self) {
        println!("BridgeProvider: Cleaning up...");
        self.inner.cleanup();
        println!("BridgeProvider: Cleanup complete!");
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

// Device bridge wrapper - mirrors the provider pattern
struct BridgeDevice {
    inner: Box<dyn crate::traits::TrackedDeviceServerDriver>,
}

impl BridgeDevice {
    fn new(device: Box<dyn crate::traits::TrackedDeviceServerDriver>) -> Self {
        Self { inner: device }
    }

    fn activate(&mut self, device_id: u32) -> bool {
        println!("BridgeDevice: Activating with device ID {}", device_id);
        match self.inner.activate(device_id) {
            Ok(()) => {
                println!("BridgeDevice: Activation successful!");
                true
            }
            Err(_) => {
                println!("BridgeDevice: Activation failed!");
                false
            }
        }
    }

    fn deactivate(&mut self) {
        println!("BridgeDevice: Deactivating...");
        self.inner.deactivate();
        println!("BridgeDevice: Deactivation complete!");
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

// Simple test provider that implements the trait
struct SimpleTestProvider {
    initialized: bool,
}

impl SimpleTestProvider {
    fn new() -> Self {
        println!("SimpleTestProvider: Creating new provider");
        Self { initialized: false }
    }
}

impl ServerTrackedDeviceProvider for SimpleTestProvider {
    fn init(&mut self, driver_context: &dyn DriverContext) -> Result<(), vr::EVRInitError> {
        println!("SimpleTestProvider: init() called");
        self.initialized = true;
        println!("SimpleTestProvider: Successfully initialized!");
        Ok(())
    }

    fn cleanup(&mut self) {
        println!("SimpleTestProvider: cleanup() called");
        self.initialized = false;
    }

    fn run_frame(&mut self) {
        // Called every frame - usually don't print here
    }

    fn should_block_standby_mode(&self) -> bool {
        false
    }

    fn enter_standby(&mut self) {
        println!("SimpleTestProvider: enter_standby() called");
    }

    fn leave_standby(&mut self) {
        println!("SimpleTestProvider: leave_standby() called");
    }
}

// C functions that the C++ bridge calls
#[unsafe(no_mangle)]
pub extern "C" fn rust_provider_create() -> *mut c_void {
    println!("rust_provider_create: Creating Rust provider...");

    let provider = unsafe {
        if let Some(factory) = PROVIDER_FACTORY {
            factory()
        } else {
            // Fallback to test provider if no factory registered
            println!("rust_provider_create: No provider factory registered, using test provider");
            Box::new(SimpleTestProvider::new())
        }
    };

    let bridge_provider = Box::new(BridgeProvider::new(provider));
    let ptr = Box::into_raw(bridge_provider) as *mut c_void;
    println!("rust_provider_create: Created provider at {:p}", ptr);
    ptr
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_provider_destroy(handle: *mut c_void) {
    println!("rust_provider_destroy: Destroying provider at {:p}", handle);
    if !handle.is_null() {
        unsafe {
            let _provider = Box::from_raw(handle as *mut BridgeProvider);
            println!("rust_provider_destroy: Provider destroyed");
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_provider_init(handle: *mut c_void, driver_context: *mut c_void) -> i32 {
    println!("rust_provider_init: Called with handle {:p}", handle);
    if handle.is_null() {
        return -1; // Error
    }

    unsafe {
        let provider = &mut *(handle as *mut BridgeProvider);
        if provider.init(driver_context) {
            0 // Success
        } else {
            -1 // Error
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_provider_cleanup(handle: *mut c_void) {
    println!("rust_provider_cleanup: Called with handle {:p}", handle);
    if !handle.is_null() {
        unsafe {
            let provider = &mut *(handle as *mut BridgeProvider);
            provider.cleanup();
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_provider_run_frame(handle: *mut c_void) {
    if !handle.is_null() {
        unsafe {
            let provider = &mut *(handle as *mut BridgeProvider);
            provider.run_frame();
        }
    }
}

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

#[unsafe(no_mangle)]
pub extern "C" fn rust_provider_enter_standby(handle: *mut c_void) {
    if !handle.is_null() {
        unsafe {
            let provider = &mut *(handle as *mut BridgeProvider);
            provider.enter_standby();
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_provider_leave_standby(handle: *mut c_void) {
    if !handle.is_null() {
        unsafe {
            let provider = &mut *(handle as *mut BridgeProvider);
            provider.leave_standby();
        }
    }
}

// Device bridge C functions that C++ will call
#[unsafe(no_mangle)]
pub extern "C" fn rust_device_create_hmd(serial_number: *const std::ffi::c_char) -> *mut c_void {
    println!("rust_device_create_hmd: Creating HMD device");

    // For now, create a simple test device
    // Later we'll make this call your actual SimpleHmdDriver
    let serial = unsafe {
        if serial_number.is_null() {
            "UNKNOWN_HMD".to_string()
        } else {
            std::ffi::CStr::from_ptr(serial_number)
                .to_str()
                .unwrap_or("INVALID_HMD")
                .to_string()
        }
    };

    // Create a simple test device for now
    let device = Box::new(SimpleTestDevice::new(serial));
    let bridge_device = Box::new(BridgeDevice::new(device));

    let ptr = Box::into_raw(bridge_device) as *mut c_void;
    println!("rust_device_create_hmd: Created device at {:p}", ptr);
    ptr
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_device_destroy(handle: *mut c_void) {
    println!("rust_device_destroy: Destroying device at {:p}", handle);
    if !handle.is_null() {
        unsafe {
            let _device = Box::from_raw(handle as *mut BridgeDevice);
            println!("rust_device_destroy: Device destroyed");
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_device_activate(handle: *mut c_void, device_id: u32) -> i32 {
    println!(
        "rust_device_activate: Called with handle {:p}, device_id {}",
        handle, device_id
    );
    if handle.is_null() {
        return -1;
    }

    unsafe {
        let device = &mut *(handle as *mut BridgeDevice);
        if device.activate(device_id) {
            0 // Success
        } else {
            -1 // Error
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_device_deactivate(handle: *mut c_void) {
    println!("rust_device_deactivate: Called with handle {:p}", handle);
    if !handle.is_null() {
        unsafe {
            let device = &mut *(handle as *mut BridgeDevice);
            device.deactivate();
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_device_run_frame(handle: *mut c_void) {
    if !handle.is_null() {
        unsafe {
            let device = &mut *(handle as *mut BridgeDevice);
            device.run_frame();
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_device_enter_standby(handle: *mut c_void) {
    if !handle.is_null() {
        unsafe {
            let device = &mut *(handle as *mut BridgeDevice);
            device.enter_standby();
        }
    }
}

// Simple test device for now
struct SimpleTestDevice {
    serial: String,
    device_id: Option<u32>,
}

impl SimpleTestDevice {
    fn new(serial: String) -> Self {
        println!("SimpleTestDevice: Creating device with serial '{}'", serial);
        Self {
            serial,
            device_id: None,
        }
    }
}

impl crate::traits::TrackedDeviceServerDriver for SimpleTestDevice {
    fn activate(&mut self, device_id: vr::TrackedDeviceIndex_t) -> Result<(), vr::EVRInitError> {
        println!(
            "SimpleTestDevice: activate() called with device_id {}",
            device_id
        );
        self.device_id = Some(device_id);
        Ok(())
    }

    fn deactivate(&mut self) {
        println!("SimpleTestDevice: deactivate() called");
        self.device_id = None;
    }

    fn run_frame(&mut self) {
        // Called every frame - don't print
    }

    fn get_serial_number(&self) -> String {
        self.serial.clone()
    }

    fn enter_standby(&mut self) {
        println!("SimpleTestDevice: enter_standby() called");
    }

    fn get_device_class(&self) -> vr::ETrackedDeviceClass {
        vr::ETrackedDeviceClass::TrackedDeviceClass_HMD
    }
}

// Export the factory function
unsafe extern "C" {
    fn create_rust_server_provider() -> *mut c_void;
}

pub fn create_provider_wrapper() -> *mut c_void {
    unsafe { create_rust_server_provider() }
}

// Public wrapper functions for device bridge - these just expose the extern and implementation functions
pub unsafe fn create_rust_device_wrapper_public(rust_device: *mut c_void) -> *mut c_void {
    create_rust_device_wrapper(rust_device)
}

pub unsafe fn server_driver_host_tracked_device_added_public(
    host_ptr: *mut c_void,
    serial_number: *const std::ffi::c_char,
    device_class: i32,
    device_driver_ptr: *mut c_void,
) -> bool {
    server_driver_host_tracked_device_added(host_ptr, serial_number, device_class, device_driver_ptr)
}

// Expose the device creation function by calling it directly
pub fn rust_device_create_hmd_public(serial_number: *const std::ffi::c_char) -> *mut c_void {
    unsafe { rust_device_create_hmd(serial_number) }
}
