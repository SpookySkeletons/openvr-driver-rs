//! Tracked Device Server Driver interface
//!
//! This interface represents a tracked device (HMD, controller, tracker, etc.)
//! in the OpenVR system.

use super::ComponentResult;
use crate::{DriverPose, DriverResult, InitError};
use std::sync::{Arc, Mutex};

/// Interface for tracked devices in OpenVR
///
/// Implement this trait for any device you want to expose to SteamVR.
/// This includes HMDs, controllers, trackers, and other tracked devices.
///
/// # Example
///
/// ```no_run
/// use openvr_driver::prelude::*;
/// use std::sync::Arc;
///
/// struct MyHmdDevice {
///     serial_number: String,
///     device_index: u32,
///     display_component: Arc<MyDisplayComponent>,
/// }
///
/// impl TrackedDeviceServerDriver for MyHmdDevice {
///     fn activate(&mut self, device_index: u32) -> DriverResult<()> {
///         self.device_index = device_index;
///         // Initialize hardware, start tracking, etc.
///         Ok(())
///     }
///
///     fn deactivate(&mut self) {
///         // Stop tracking, release hardware, etc.
///     }
///
///     fn get_component(&self, component_name: &str) -> ComponentResult {
///         match component_name {
///             "IVRDisplayComponent" => {
///                 Some(self.display_component.clone().create_vtable())
///             }
///             _ => None,
///         }
///     }
///
///     fn get_pose(&self) -> DriverPose {
///         // Return current device pose
///         DriverPose::default()
///     }
/// }
/// ```
pub trait TrackedDeviceServerDriver: Send + Sync + 'static {
    /// Get the serial number of this device
    ///
    /// This is used to identify the device uniquely in OpenVR.
    /// It should be a stable identifier that doesn't change between sessions.
    fn get_serial_number(&self) -> String {
        "GENERIC_DEVICE_001".to_string()
    }

    /// Activate the device
    ///
    /// Called when OpenVR assigns a device index to this device.
    /// This is where you should initialize hardware, start tracking,
    /// and set up device properties.
    ///
    /// Note: Due to Arc<dyn Trait> limitations, this may not be called directly
    /// from the vtable. Implementations should use interior mutability or
    /// call `perform_activation()` after registration.
    ///
    /// # Arguments
    /// * `device_index` - The device index assigned by OpenVR
    ///
    /// # Returns
    /// * `Ok(())` if activation succeeded
    /// * `Err(InitError)` if activation failed
    fn activate(&mut self, device_index: u32) -> DriverResult<()>;

    /// Perform self-activation after registration
    ///
    /// This method can be called by the device implementation after it has been
    /// registered with OpenVR to perform activation tasks. This works around
    /// the limitation of not being able to call mutable methods through Arc<dyn>.
    ///
    /// # Arguments
    /// * `device_index` - The device index to activate with (usually 0 for first HMD)
    ///
    /// # Returns
    /// * `Ok(())` if activation succeeded
    /// * `Err(InitError)` if activation failed
    fn perform_activation(&self, device_index: u32) -> DriverResult<()> {
        // Default implementation does nothing
        // Override this in implementations that use interior mutability
        Ok(())
    }

    /// Deactivate the device
    ///
    /// Called when the device is being removed or OpenVR is shutting down.
    /// Stop tracking, release hardware resources, and clean up.
    fn deactivate(&mut self);

    /// Called when entering standby mode
    ///
    /// Put the device into a low-power state if possible.
    fn enter_standby(&mut self) {}

    /// Get a component interface from this device
    ///
    /// Components provide specific functionality like display output,
    /// controller input, camera access, etc.
    ///
    /// # Arguments
    /// * `component_name` - The name of the component interface requested
    ///
    /// # Returns
    /// * `Some(ptr)` - Raw pointer to the component vtable
    /// * `None` - If this device doesn't provide the requested component
    ///
    /// # Common Component Names
    /// * `"IVRDisplayComponent"` - For HMDs
    /// * `"IVRControllerComponent"` - For controllers
    /// * `"IVRCameraComponent"` - For devices with cameras
    /// * `"IVRDriverDirectModeComponent"` - For direct mode rendering
    fn get_component(&self, component_name: &str) -> ComponentResult {
        None
    }

    /// Handle debug requests
    ///
    /// This method is called when debugging tools send requests to your device.
    /// You can use this to implement custom debug commands.
    ///
    /// # Arguments
    /// * `request` - The debug request string
    ///
    /// # Returns
    /// * A string response to the debug request
    fn debug_request(&mut self, request: &str) -> String {
        format!("Unknown debug request: {}", request)
    }

    /// Get the current pose of the device
    ///
    /// This method is called frequently (potentially every frame) to get
    /// the current position and orientation of the device.
    ///
    /// # Returns
    /// * The current `DriverPose` for this device
    fn get_pose(&self) -> DriverPose;
}

/// Extension trait for device implementations
pub trait TrackedDeviceServerDriverExt: TrackedDeviceServerDriver {
    /// Helper to create a default pose
    fn create_default_pose() -> DriverPose {
        DriverPose {
            poseTimeOffset: 0.0,
            qWorldFromDriverRotation: crate::HmdQuaternion {
                w: 1.0,
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            vecWorldFromDriverTranslation: [0.0, 0.0, 0.0],
            qDriverFromHeadRotation: crate::HmdQuaternion {
                w: 1.0,
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            vecDriverFromHeadTranslation: [0.0, 0.0, 0.0],
            vecPosition: [0.0, 0.0, 0.0],
            vecVelocity: [0.0, 0.0, 0.0],
            vecAcceleration: [0.0, 0.0, 0.0],
            qRotation: crate::HmdQuaternion {
                w: 1.0,
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            vecAngularVelocity: [0.0, 0.0, 0.0],
            vecAngularAcceleration: [0.0, 0.0, 0.0],
            result: unsafe { std::mem::transmute(200i32) }, // TrackingResult_Running_OK
            poseIsValid: true,
            willDriftInYaw: false,
            shouldApplyHeadModel: false,
            deviceIsConnected: true,
        }
    }

    /// Helper to create a pose at a specific position
    fn create_pose_at(x: f64, y: f64, z: f64) -> DriverPose {
        let mut pose = Self::create_default_pose();
        pose.vecPosition = [x, y, z];
        pose
    }

    /// Helper to create a pose with rotation
    fn create_pose_with_rotation(w: f64, x: f64, y: f64, z: f64) -> DriverPose {
        let mut pose = Self::create_default_pose();
        pose.qRotation = crate::HmdQuaternion { w, x, y, z };
        pose
    }
}

impl<T: TrackedDeviceServerDriver> TrackedDeviceServerDriverExt for T {}

/// Wrapper for devices that need interior mutability for OpenVR callbacks
///
/// Since OpenVR's C++ vtables expect mutable methods but we have Arc<dyn Trait>,
/// we need to wrap devices with interior mutability to handle the mutable calls.
pub struct DeviceWrapper<T: TrackedDeviceServerDriver> {
    inner: Arc<Mutex<T>>,
}

impl<T: TrackedDeviceServerDriver> DeviceWrapper<T> {
    /// Create a new device wrapper
    pub fn new(device: T) -> Self {
        Self {
            inner: Arc::new(Mutex::new(device)),
        }
    }

    /// Get the inner device reference
    pub fn inner(&self) -> &Arc<Mutex<T>> {
        &self.inner
    }
}

impl<T: TrackedDeviceServerDriver + 'static> TrackedDeviceServerDriver for DeviceWrapper<T> {
    fn activate(&mut self, device_index: u32) -> DriverResult<()> {
        self.inner.lock().unwrap().activate(device_index)
    }

    fn deactivate(&mut self) {
        self.inner.lock().unwrap().deactivate()
    }

    fn enter_standby(&mut self) {
        self.inner.lock().unwrap().enter_standby()
    }

    fn get_component(&self, component_name: &str) -> ComponentResult {
        self.inner.lock().unwrap().get_component(component_name)
    }

    fn debug_request(&mut self, request: &str) -> String {
        self.inner.lock().unwrap().debug_request(request)
    }

    fn get_pose(&self) -> DriverPose {
        self.inner.lock().unwrap().get_pose()
    }
}
