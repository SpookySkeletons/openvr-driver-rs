//! Camera Component interface
//!
//! This interface is implemented by devices that provide camera/passthrough functionality.

use super::CameraConfiguration;
use crate::DriverResult;

/// Camera component for devices with camera capabilities
///
/// Implement this trait to provide camera/passthrough functionality for your device.
/// This is typically used for AR passthrough or external camera tracking.
pub trait CameraComponent: Send + Sync + 'static {
    /// Get the number of cameras available
    fn get_camera_count(&self) -> u32 {
        0
    }

    /// Get camera configuration
    ///
    /// # Arguments
    /// * `camera_index` - Index of the camera (0-based)
    ///
    /// # Returns
    /// * Camera configuration if the index is valid
    fn get_camera_configuration(&self, camera_index: u32) -> Option<CameraConfiguration> {
        None
    }

    /// Start camera streaming
    ///
    /// # Arguments
    /// * `camera_index` - Index of the camera to start
    fn start_streaming(&mut self, camera_index: u32) -> DriverResult<()> {
        Ok(())
    }

    /// Stop camera streaming
    ///
    /// # Arguments
    /// * `camera_index` - Index of the camera to stop
    fn stop_streaming(&mut self, camera_index: u32) -> DriverResult<()> {
        Ok(())
    }

    /// Get the current camera frame
    ///
    /// # Arguments
    /// * `camera_index` - Index of the camera
    /// * `buffer` - Buffer to write frame data into
    ///
    /// # Returns
    /// * Number of bytes written to the buffer
    fn get_frame(&self, camera_index: u32, buffer: &mut [u8]) -> usize {
        0
    }

    /// Check if a camera is currently streaming
    ///
    /// # Arguments
    /// * `camera_index` - Index of the camera
    fn is_streaming(&self, camera_index: u32) -> bool {
        false
    }
}
