//! High-level interface definitions for OpenVR drivers
//!
//! These traits define the behavior that drivers must implement.
//! All low-level vtable generation is handled automatically by the library.

mod camera;
mod controller;
mod device;
mod display;
mod driver_input;
mod provider;
mod virtual_display;
mod watchdog;

pub use camera::CameraComponent;
pub use controller::ControllerComponent;
pub use device::TrackedDeviceServerDriver;
pub use display::DisplayComponent;
pub use display::Eye;
pub use driver_input::DriverInput;
pub use provider::ServerTrackedDeviceProvider;
pub use virtual_display::VirtualDisplay;
pub use watchdog::WatchdogProvider;

use crate::sys::root::vr;
use std::ffi::c_void;

/// Result type for component retrieval
pub type ComponentResult = Option<*mut c_void>;

/// Helper trait for components that can be retrieved from devices
pub trait Component: Send + Sync {
    /// Get the component name for OpenVR
    fn component_name() -> &'static str;

    /// Create a vtable wrapper for this component
    fn create_vtable(self: std::sync::Arc<Self>) -> *mut c_void;
}

/// Implementation of Component for DisplayComponent
impl<T: DisplayComponent + 'static> Component for T {
    fn component_name() -> &'static str {
        "IVRDisplayComponent"
    }

    fn create_vtable(self: std::sync::Arc<Self>) -> *mut c_void {
        crate::vtables::create_display_vtable(self)
    }
}

/// Device activation state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivationState {
    /// Device is not activated
    Inactive,
    /// Device is activating
    Activating,
    /// Device is active
    Active,
    /// Device is deactivating
    Deactivating,
}

/// Common device properties that most drivers will need
#[derive(Debug, Clone)]
pub struct DeviceProperties {
    pub model_number: String,
    pub serial_number: String,
    pub render_model_name: String,
    pub manufacturer_name: String,
    pub tracking_system_name: String,
    pub hardware_revision: String,
    pub firmware_version: String,
    pub tracking_firmware_version: String,
}

impl Default for DeviceProperties {
    fn default() -> Self {
        Self {
            model_number: "Generic Model".to_string(),
            serial_number: "GENERIC_001".to_string(),
            render_model_name: "generic_hmd".to_string(),
            manufacturer_name: "Generic Manufacturer".to_string(),
            tracking_system_name: "lighthouse".to_string(),
            hardware_revision: "1.0".to_string(),
            firmware_version: "1.0.0".to_string(),
            tracking_firmware_version: "1.0.0".to_string(),
        }
    }
}

/// Display configuration for HMD devices
#[derive(Debug, Clone)]
pub struct DisplayConfiguration {
    /// Window position X
    pub window_x: i32,
    /// Window position Y
    pub window_y: i32,
    /// Window width
    pub window_width: i32,
    /// Window height
    pub window_height: i32,
    /// Render target width
    pub render_width: i32,
    /// Render target height
    pub render_height: i32,
    /// Display frequency in Hz
    pub display_frequency: f32,
    /// Inter-pupillary distance in meters
    pub ipd: f32,
    /// Seconds from vsync to photons
    pub seconds_from_vsync_to_photons: f32,
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
            display_frequency: 90.0,
            ipd: 0.063,
            seconds_from_vsync_to_photons: 0.011,
        }
    }
}

/// Controller configuration
#[derive(Debug, Clone)]
pub struct ControllerConfiguration {
    pub role: ControllerRole,
    pub has_haptics: bool,
    pub has_trackpad: bool,
    pub has_joystick: bool,
    pub has_trigger: bool,
    pub has_grip: bool,
    pub button_count: u32,
}

/// Controller role (left/right hand, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControllerRole {
    LeftHand,
    RightHand,
    Tracker,
    Other,
}

/// Camera configuration
#[derive(Debug, Clone)]
pub struct CameraConfiguration {
    pub width: u32,
    pub height: u32,
    pub frame_rate: f32,
    pub exposure_time: f32,
    pub gain: f32,
}

// Re-export types from sys that are commonly used
pub use vr::{
    DriverPose_t as DriverPose, HmdMatrix34_t as HmdMatrix34, HmdQuaternion_t as HmdQuaternion,
    HmdVector3_t as HmdVector3,
};
