//! # OpenVR Driver Rust Bindings
//!
//! Minimal Rust bindings for creating OpenVR drivers. This crate provides
//! the essential bridge between Rust code and OpenVR's C++ interface system.
//!
//! ## Usage
//!
//! Add this to your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! openvr-driver-rs = { path = "../openvr-driver-rs" }
//! ```
//!
//! Then implement the required traits:
//! ```rust
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

// ============================================================================
// CORE RE-EXPORTS
// ============================================================================

// Re-export the entire sys crate for low-level access
pub use openvr_driver_sys as sys;

// Essential OpenVR types needed for driver implementation
pub use openvr_driver_sys::vr::{
    ETrackedDeviceClass,
    EVRInitError, 
    TrackedDeviceIndex_t,
    DriverHandle_t,
};

// Core traits that drivers must implement
pub use openvr_driver_sys::traits::{
    ServerTrackedDeviceProvider,
    TrackedDeviceServerDriver, 
    DriverContext,
};

// Bridge functions for connecting Rust to OpenVR
pub use openvr_driver_sys::bridge::{
    create_device_bridge,
    create_cpp_device_wrapper,
    register_device_with_openvr,
    register_provider_factory,
    create_provider_wrapper,
};

// ============================================================================
// TYPE ALIASES
// ============================================================================

/// Result type for driver operations that can fail with OpenVR errors
pub type DriverResult<T> = Result<T, EVRInitError>;