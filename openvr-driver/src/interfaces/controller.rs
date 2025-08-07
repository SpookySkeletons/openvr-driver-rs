//! Controller Component interface
//!
//! This interface is implemented by controller devices to provide
//! input and haptic feedback functionality.

use super::ControllerConfiguration;
use crate::{ControllerState, DriverResult, InputComponentHandle};

/// Controller component for input devices
///
/// Implement this trait to provide controller functionality for your device.
/// This includes button states, analog inputs, and haptic feedback.
pub trait ControllerComponent: Send + Sync + 'static {
    /// Get the current controller state
    ///
    /// Returns the current state of all buttons, triggers, and analog inputs.
    fn get_controller_state(&self) -> ControllerState;

    /// Trigger haptic feedback
    ///
    /// # Arguments
    /// * `duration_seconds` - How long the haptic pulse should last
    /// * `frequency` - Frequency of the haptic pulse in Hz
    /// * `amplitude` - Amplitude of the haptic pulse (0.0 to 1.0)
    fn trigger_haptic_pulse(&mut self, duration_seconds: f32, frequency: f32, amplitude: f32) {
        // Default implementation does nothing
        // Override to provide haptic feedback
    }

    /// Update the controller state
    ///
    /// Called each frame to update the controller's input state.
    fn update_controller_state(&mut self) {
        // Default implementation does nothing
    }

    /// Get the controller configuration
    ///
    /// Returns information about the controller's capabilities.
    fn get_configuration(&self) -> ControllerConfiguration;

    /// Register an input component
    ///
    /// # Arguments
    /// * `name` - The name of the input component
    /// * `handle` - The handle assigned by OpenVR
    fn register_input_component(&mut self, name: &str, handle: InputComponentHandle) {
        // Default implementation does nothing
    }
}
