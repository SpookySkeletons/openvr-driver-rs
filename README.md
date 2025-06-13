# OpenVR Driver Rust

Minimal Rust bindings for creating OpenVR drivers. This crate provides the essential bridge between Rust code and OpenVR's C++ interface system.

## Overview

OpenVR drivers are dynamic libraries that extend SteamVR functionality by adding support for VR hardware devices. Since OpenVR expects C++ virtual interfaces and Rust cannot directly implement C++ virtual functions, this crate provides a bridge system that:

- Exposes clean Rust traits for driver implementation
- Handles the C++/Rust interoperability automatically
- Provides type-safe access to OpenVR APIs

## Usage

Create a new driver project and add the dependency:

```bash
# Create your driver project
cargo new my-vr-driver --lib
cd my-vr-driver

# Add the OpenVR driver dependency
cargo add openvr-driver-rs --git https://github.com/SpookySkeletons/openvr-driver-rs

# Configure as dynamic library in Cargo.toml
```

Then update your `Cargo.toml` to set the crate type:

```toml
[lib]
crate-type = ["cdylib"]  # Required for OpenVR drivers
```

## Basic Driver Structure

```rust
use openvr_driver_rs::*;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::ptr;

// 1. Implement the device provider (manages driver lifecycle)
struct MyDriverProvider {
    devices: Vec<MyDevice>,
    driver_host: Option<*mut sys::vr::IVRServerDriverHost>,
}

impl ServerTrackedDeviceProvider for MyDriverProvider {
    fn init(&mut self, context: &dyn DriverContext) -> DriverResult<()> {
        // Initialize your driver and register devices
        Ok(())
    }

    fn cleanup(&mut self) {
        // Clean up resources
    }

    fn run_frame(&mut self) {
        // Called every frame - update device states here
    }

    // ... other required methods
}

// 2. Implement your device(s)
struct MyDevice {
    device_id: Option<TrackedDeviceIndex_t>,
}

impl TrackedDeviceServerDriver for MyDevice {
    fn activate(&mut self, device_id: TrackedDeviceIndex_t) -> DriverResult<()> {
        self.device_id = Some(device_id);
        Ok(())
    }

    fn get_serial_number(&self) -> String {
        "MY_DEVICE_001".to_string()
    }

    fn get_device_class(&self) -> ETrackedDeviceClass {
        ETrackedDeviceClass::TrackedDeviceClass_HMD
    }

    // ... other required methods
}

// 3. Export the driver factory function
#[unsafe(no_mangle)]
pub unsafe extern "C" fn HmdDriverFactory(
    interface_name: *const c_char,
    return_code: *mut sys::vr::EVRInitError,
) -> *mut c_void {
    // Handle OpenVR interface requests
    // See full example for complete implementation
}
```

## Device Registration

To register devices with OpenVR:

```rust
fn register_device(&self, device: &MyDevice, host: *mut sys::vr::IVRServerDriverHost) -> DriverResult<()> {
    let serial = CString::new(device.get_serial_number()).unwrap();
    let bridge_device = Box::new(MyDevice::new());

    // Create bridge wrapper
    let rust_device_bridge = create_device_bridge(bridge_device);
    let cpp_device = unsafe { create_cpp_device_wrapper(rust_device_bridge) };

    // Register with OpenVR
    let success = unsafe {
        register_device_with_openvr(
            host as *mut std::ffi::c_void,
            serial.as_ptr(),
            device.get_device_class(),
            cpp_device,
        )
    };

    if success { Ok(()) } else { Err(/* error */) }
}
```

## Building

For native builds:
```bash
cargo build --release
```

For Windows cross-compilation from Linux:
```bash
# Install cross-compilation tools
cargo install cross
rustup target add x86_64-pc-windows-gnu

# Build for Windows
cross build --target x86_64-pc-windows-gnu --release
```

## Deployment

1. Build your driver as a `.dll` (Windows) or `.so` (Linux)
2. Create an OpenVR driver manifest (`driver.vrdrivermanifest`)
3. Place in SteamVR's `drivers` directory
4. Register with SteamVR

## Architecture

This crate consists of three layers:

- **openvr-driver-rs**: High-level Rust API (this crate)
- **openvr-driver-sys**: Low-level bindings and C++ bridge
- **C++ Bridge**: Implements OpenVR interfaces and forwards to Rust

The bridge handles all the complexity of converting between Rust traits and OpenVR's C++ virtual interface system.

## License

Licensed under either of:
- Apache License, Version 2.0
- MIT License

at your option.
