//! Driver Input interface
//!
//! This interface provides methods for creating and managing input components
//! like buttons, triggers, joysticks, and haptics.

use crate::{DriverResult, InputComponentHandle, PropertyHandle};
use std::ffi::c_void;

/// Driver input interface for managing input components
///
/// This trait provides methods to create and update various input components
/// that can be used by tracked devices like controllers and trackers.
pub trait DriverInput: Send + Sync + 'static {
    /// Create a boolean input component (button)
    ///
    /// # Arguments
    /// * `device_handle` - Handle to the device
    /// * `name` - Name of the input component (e.g., "/input/trigger/click")
    /// * `handle` - Output handle for the created component
    fn create_boolean_component(
        &mut self,
        device_handle: PropertyHandle,
        name: &str,
    ) -> DriverResult<InputComponentHandle> {
        Ok(0)
    }

    /// Create a scalar input component (analog input like trigger or joystick axis)
    ///
    /// # Arguments
    /// * `device_handle` - Handle to the device
    /// * `name` - Name of the input component (e.g., "/input/trigger/value")
    /// * `handle` - Output handle for the created component
    fn create_scalar_component(
        &mut self,
        device_handle: PropertyHandle,
        name: &str,
    ) -> DriverResult<InputComponentHandle> {
        Ok(0)
    }

    /// Create a haptic component
    ///
    /// # Arguments
    /// * `device_handle` - Handle to the device
    /// * `name` - Name of the haptic component (e.g., "/output/haptic")
    /// * `handle` - Output handle for the created component
    fn create_haptic_component(
        &mut self,
        device_handle: PropertyHandle,
        name: &str,
    ) -> DriverResult<InputComponentHandle> {
        Ok(0)
    }

    /// Update a boolean component's state
    ///
    /// # Arguments
    /// * `handle` - Handle to the component
    /// * `value` - New boolean value
    /// * `time_offset` - Time offset in seconds (0 for current time)
    fn update_boolean_component(
        &mut self,
        handle: InputComponentHandle,
        value: bool,
        time_offset: f64,
    ) -> DriverResult<()> {
        Ok(())
    }

    /// Update a scalar component's value
    ///
    /// # Arguments
    /// * `handle` - Handle to the component
    /// * `value` - New scalar value (typically 0.0 to 1.0)
    /// * `time_offset` - Time offset in seconds (0 for current time)
    fn update_scalar_component(
        &mut self,
        handle: InputComponentHandle,
        value: f32,
        time_offset: f64,
    ) -> DriverResult<()> {
        Ok(())
    }
}
