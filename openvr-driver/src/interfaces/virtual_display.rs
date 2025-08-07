//! Virtual Display interface
//!
//! This interface is implemented by virtual display drivers that don't
//! correspond to physical hardware.

use crate::DriverResult;

/// Virtual display interface for software-only displays
///
/// Implement this trait to create a virtual display that doesn't correspond
/// to physical hardware. This is useful for streaming, recording, or testing.
pub trait VirtualDisplay: Send + Sync + 'static {
    /// Present a frame to the virtual display
    ///
    /// # Arguments
    /// * `sync_texture` - Handle to the texture to present
    /// * `bounds` - The bounds of the texture to present
    fn present(&mut self, sync_texture: *mut std::ffi::c_void, bounds: &VirtualDisplayBounds);

    /// Wait for the next frame to be presented
    ///
    /// This method should block until it's time to present the next frame,
    /// based on the virtual display's refresh rate.
    fn wait_for_present(&mut self) {
        // Default implementation doesn't wait
    }

    /// Get the timing information for the virtual display
    fn get_timing_info(&self) -> VirtualDisplayTiming {
        VirtualDisplayTiming::default()
    }
}

/// Bounds for virtual display presentation
#[derive(Debug, Clone, Copy)]
pub struct VirtualDisplayBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Timing information for virtual displays
#[derive(Debug, Clone, Copy)]
pub struct VirtualDisplayTiming {
    /// Frame duration in seconds
    pub frame_duration: f64,
    /// Time until next vsync in seconds
    pub time_since_last_vsync: f64,
    /// Display frequency in Hz
    pub display_frequency: f32,
}

impl Default for VirtualDisplayTiming {
    fn default() -> Self {
        Self {
            frame_duration: 1.0 / 90.0, // 90 Hz default
            time_since_last_vsync: 0.0,
            display_frequency: 90.0,
        }
    }
}
