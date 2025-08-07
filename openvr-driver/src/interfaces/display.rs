//! Display Component interface
//!
//! This interface is implemented by HMD devices to provide display
//! configuration and rendering parameters to OpenVR.

use super::DisplayConfiguration;
use crate::DriverResult;

/// Display component for HMD devices
///
/// Implement this trait to provide display configuration for your HMD.
/// This component tells OpenVR about your display's resolution, field of view,
/// distortion parameters, and other rendering properties.
///
/// # Example
///
/// ```no_run
/// use openvr_driver::prelude::*;
///
/// struct MyDisplayComponent {
///     config: DisplayConfiguration,
/// }
///
/// impl DisplayComponent for MyDisplayComponent {
///     fn get_window_bounds(&self) -> (i32, i32, i32, i32) {
///         (
///             self.config.window_x,
///             self.config.window_y,
///             self.config.window_width,
///             self.config.window_height,
///         )
///     }
///
///     fn get_recommended_render_target_size(&self) -> (u32, u32) {
///         (
///             self.config.render_width as u32,
///             self.config.render_height as u32,
///         )
///     }
///
///     fn get_eye_to_head_transform(&self, eye: Eye) -> HmdMatrix34 {
///         // Return the transform from eye space to head space
///         let ipd_half = self.config.ipd / 2.0;
///         let x_offset = match eye {
///             Eye::Left => -ipd_half,
///             Eye::Right => ipd_half,
///         };
///
///         HmdMatrix34 {
///             m: [
///                 [1.0, 0.0, 0.0, x_offset],
///                 [0.0, 1.0, 0.0, 0.0],
///                 [0.0, 0.0, 1.0, 0.0],
///             ],
///         }
///     }
/// }
/// ```
pub trait DisplayComponent: Send + Sync + 'static {
    /// Get the window bounds for the display
    ///
    /// Returns (x, y, width, height) of the display window on the desktop.
    /// For direct mode displays, this typically returns the display's native resolution.
    ///
    /// # Returns
    /// * `(x, y, width, height)` - Window position and size
    fn get_window_bounds(&self) -> (i32, i32, i32, i32);

    /// Get the recommended render target size
    ///
    /// This is the resolution that applications should render at for each eye.
    /// It may be different from the window bounds to account for distortion correction.
    ///
    /// # Returns
    /// * `(width, height)` - Recommended render resolution per eye
    fn get_recommended_render_target_size(&self) -> (u32, u32);

    /// Get the transform from eye space to head space
    ///
    /// This transform positions each eye relative to the head center.
    /// It's primarily used to implement the interpupillary distance (IPD).
    ///
    /// # Arguments
    /// * `eye` - Which eye to get the transform for
    ///
    /// # Returns
    /// * The transformation matrix from eye space to head space
    fn get_eye_to_head_transform(&self, eye: Eye) -> crate::HmdMatrix34;

    /// Get the projection matrix for an eye
    ///
    /// # Arguments
    /// * `eye` - Which eye to get the projection for
    /// * `near_plane` - Near clipping plane distance
    /// * `far_plane` - Far clipping plane distance
    ///
    /// # Returns
    /// * The projection matrix for the specified eye
    fn get_projection(&self, eye: Eye, near_plane: f32, far_plane: f32) -> crate::HmdMatrix34 {
        // Default implementation provides a simple projection
        // Override this for custom FOV or projection parameters
        let aspect = 1.0; // Assuming square aspect for simplicity
        let fov = 90.0_f32.to_radians();
        let half_fov = fov / 2.0;

        let left = -near_plane * half_fov.tan();
        let right = near_plane * half_fov.tan();
        let top = near_plane * half_fov.tan() / aspect;
        let bottom = -near_plane * half_fov.tan() / aspect;

        // Build projection matrix
        let a = (right + left) / (right - left);
        let b = (top + bottom) / (top - bottom);
        let c = -(far_plane + near_plane) / (far_plane - near_plane);
        let d = -(2.0 * far_plane * near_plane) / (far_plane - near_plane);

        crate::HmdMatrix34 {
            m: [
                [2.0 * near_plane / (right - left), 0.0, a, 0.0],
                [0.0, 2.0 * near_plane / (top - bottom), b, 0.0],
                [0.0, 0.0, c, d],
            ],
        }
    }

    /// Get distortion coordinates for color channels
    ///
    /// This method computes the distorted coordinates for each color channel
    /// to correct for lens distortion and chromatic aberration.
    ///
    /// # Arguments
    /// * `eye` - Which eye
    /// * `u` - Horizontal coordinate (0.0 to 1.0)
    /// * `v` - Vertical coordinate (0.0 to 1.0)
    ///
    /// # Returns
    /// * `(red_u, red_v, green_u, green_v, blue_u, blue_v)` - Distorted coordinates for each color
    fn compute_distortion(&self, eye: Eye, u: f32, v: f32) -> (f32, f32, f32, f32, f32, f32) {
        // Default: no distortion
        (u, v, u, v, u, v)
    }

    /// Check if the display is on the desktop
    ///
    /// Returns true if this is an extended mode display (appears as a monitor).
    /// Returns false if this is a direct mode display.
    ///
    /// # Returns
    /// * `true` if the display is on the desktop
    /// * `false` if the display is in direct mode
    fn is_display_on_desktop(&self) -> bool {
        true // Default to extended mode
    }

    /// Check if the display is real (physical hardware)
    ///
    /// # Returns
    /// * `true` if this is a real physical display
    /// * `false` if this is a virtual/null display
    fn is_display_real(&self) -> bool {
        true
    }
}

/// Eye identifier for stereo rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Eye {
    Left = 0,
    Right = 1,
}

impl From<i32> for Eye {
    fn from(value: i32) -> Self {
        match value {
            0 => Eye::Left,
            1 => Eye::Right,
            _ => Eye::Left, // Default to left for invalid values
        }
    }
}
