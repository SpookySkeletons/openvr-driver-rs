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

impl DriverContext for BridgeDriverContext {
    fn get_generic_interface(&self, interface_version: &str) -> Option<*mut std::ffi::c_void> {
        // TODO: This would need to call into the actual IVRDriverContext
        // For now, return None to keep it simple
        println!(
            "BridgeDriverContext: get_generic_interface called for '{}'",
            interface_version
        );
        None
    }

    fn get_driver_handle(&self) -> vr::DriverHandle_t {
        // TODO: This would need to call into the actual IVRDriverContext
        0
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

    // TODO: Replace this with your actual SimpleHmdProvider
    // For now using test provider until we can import your provider
    let test_provider = Box::new(SimpleTestProvider::new());
    let bridge_provider = Box::new(BridgeProvider::new(test_provider));

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

// Export the factory function
unsafe extern "C" {
    fn create_rust_server_provider() -> *mut c_void;
}

pub fn create_provider_wrapper() -> *mut c_void {
    unsafe { create_rust_server_provider() }
}
