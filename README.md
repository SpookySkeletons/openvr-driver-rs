# OpenVR Driver Rust Bindings

Pure Rust bindings for creating OpenVR device drivers. This crate provides a safe, idiomatic Rust interface for implementing OpenVR drivers without any C++ dependencies.

## Features

- **Pure Rust**: No C++ bridge or wrapper code required
- **Type-safe**: Leverages Rust's type system for safety
- **Zero-overhead**: Direct vtable implementation matches C++ ABI
- **Automatic binding generation**: Uses bindgen to stay up-to-date with OpenVR headers

## Architecture

This crate uses a similar approach to XRizer, directly implementing C++ virtual interfaces in Rust by:
- Parsing OpenVR's C++ headers with bindgen
- Generating Rust structs that match C++ vtable layouts
- Using procedural macros to generate boilerplate for interface implementations
- Providing safe Rust traits that map to OpenVR's C++ interfaces

## Project Structure

```
openvr-driver-rs/
├── src/                        # Main library with high-level traits
├── openvr-driver-bindings/     # Low-level bindgen-generated bindings
│   ├── headers/               # OpenVR C++ headers
│   └── src/                   # Generated bindings and vtable infrastructure
├── driver-macros/             # Procedural macros for interface implementation
└── examples/
    └── pure_rust_driver/      # Example driver implementation
```

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
openvr-driver-rs = { git = "https://github.com/SpookySkeletons/openvr-driver-rs" }
```

Then implement a basic driver:

```rust
use openvr_driver_rs::*;

struct MyProvider {
    // Your provider state
}

impl ServerTrackedDeviceProvider for MyProvider {
    fn init(&mut self, driver_context: *mut c_void) -> Result<(), EVRInitError> {
        // Initialize your driver
        Ok(())
    }

    fn cleanup(&mut self) {
        // Cleanup resources
    }

    fn run_frame(&mut self) {
        // Called every frame
    }

    // ... implement other required methods
}

struct MyDevice {
    // Your device state
}

impl TrackedDeviceServerDriver for MyDevice {
    fn activate(&mut self, device_index: u32) -> Result<(), EVRInitError> {
        // Activate your device
        Ok(())
    }

    fn get_pose(&self) -> DriverPose_t {
        // Return current device pose
        DriverPose_t::default()
    }

    // ... implement other required methods
}

// Use the macro to generate the entry point
openvr_driver_entry!(MyProvider);
```

## Building

Build your driver as a shared library:

```toml
[lib]
crate-type = ["cdylib"]
```

Then build with:

```bash
cargo build --release
```

The resulting `.so` (Linux), `.dll` (Windows), or `.dylib` (macOS) can be loaded by OpenVR/SteamVR.

## Status

This is a work in progress. The following components are implemented:

- [x] Bindgen-based C++ header parsing
- [x] Basic vtable structure generation
- [x] Procedural macro for interface implementation
- [ ] Complete vtable wiring and FFI boundaries
- [ ] Full trait implementations for all driver interfaces
- [ ] Comprehensive examples and documentation
- [ ] Testing with SteamVR runtime

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues.

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
 * MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Acknowledgments

This project's approach is heavily inspired by [XRizer](https://github.com/Sorenon/xrizer), which pioneered the technique of implementing C++ virtual interfaces directly in Rust.