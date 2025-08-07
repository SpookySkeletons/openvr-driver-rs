//! Server Tracked Device Provider interface
//!
//! This is the main entry point for OpenVR drivers. Your driver should implement
//! this trait to provide devices to SteamVR.

use crate::{DriverContext, DriverResult, InitError};
use std::sync::Arc;

/// Main interface for OpenVR driver providers
///
/// This trait represents the entry point for your driver. OpenVR will call these
/// methods to manage your driver's lifecycle and query for devices.
///
/// # Example
///
/// ```no_run
/// use openvr_driver::prelude::*;
/// use std::sync::Arc;
///
/// struct MyDriver {
///     devices: Vec<Arc<dyn TrackedDeviceServerDriver>>,
/// }
///
/// impl ServerTrackedDeviceProvider for MyDriver {
///     fn init(&mut self, context: &mut DriverContext) -> DriverResult<()> {
///         // Initialize your driver
///         // Register devices with context.register_device()
///         Ok(())
///     }
///
///     fn cleanup(&mut self) {
///         // Clean up resources
///     }
///
///     fn run_frame(&mut self) {
///         // Update device states each frame
///     }
///
///     fn should_block_standby_mode(&self) -> bool {
///         false
///     }
///
///     fn enter_standby(&mut self) {}
///     fn leave_standby(&mut self) {}
/// }
/// ```
pub trait ServerTrackedDeviceProvider: Send + Sync + 'static {
    /// Initialize the driver
    ///
    /// This is called once when the driver is loaded. Use the provided
    /// `DriverContext` to access OpenVR interfaces and register devices.
    ///
    /// # Arguments
    /// * `context` - The driver context for accessing OpenVR interfaces
    ///
    /// # Returns
    /// * `Ok(())` if initialization succeeded
    /// * `Err(InitError)` with appropriate error code if initialization failed
    fn init(&mut self, context: &mut DriverContext) -> DriverResult<()>;

    /// Cleanup the driver
    ///
    /// Called when the driver is being unloaded. Release any resources
    /// and ensure all devices are properly deactivated.
    fn cleanup(&mut self);

    /// Called every frame by OpenVR
    ///
    /// This is where you should update device states, poll hardware,
    /// and submit new poses. This method is called from the main
    /// OpenVR thread.
    fn run_frame(&mut self);

    /// Whether the driver should block standby mode
    ///
    /// Return `true` if your driver needs to prevent the system from
    /// entering standby mode (e.g., during tracking or when devices
    /// are active).
    ///
    /// # Returns
    /// * `true` to block standby mode
    /// * `false` to allow standby mode (default)
    fn should_block_standby_mode(&self) -> bool {
        false
    }

    /// Called when the system is entering standby
    ///
    /// Use this to put devices into a low-power state or pause
    /// tracking operations.
    fn enter_standby(&mut self) {}

    /// Called when the system is leaving standby
    ///
    /// Use this to wake up devices and resume normal operations.
    fn leave_standby(&mut self) {}

    /// Get the interface versions this driver supports
    ///
    /// This is used internally by the library. Most drivers don't need
    /// to override this method.
    fn get_interface_versions(&self) -> Vec<&'static str> {
        vec![
            "IServerTrackedDeviceProvider_005",
            "ITrackedDeviceServerDriver_005",
        ]
    }
}

/// Extension trait for provider implementations
pub trait ServerTrackedDeviceProviderExt: ServerTrackedDeviceProvider {
    /// Register a device with OpenVR
    ///
    /// This is a convenience method that wraps the context's register_device method.
    fn register_device(
        &mut self,
        context: &mut DriverContext,
        device: Arc<dyn crate::TrackedDeviceServerDriver>,
    ) -> DriverResult<()> {
        context.register_device(device)
    }
}

impl<T: ServerTrackedDeviceProvider> ServerTrackedDeviceProviderExt for T {}
