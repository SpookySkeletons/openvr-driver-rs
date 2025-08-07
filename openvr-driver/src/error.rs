//! Error handling for OpenVR drivers
//!
//! This module provides error types and result aliases for OpenVR driver operations.

use crate::sys::root::vr::EVRInitError;
use std::fmt;

/// Result type for driver operations
pub type DriverResult<T> = Result<T, DriverError>;

/// Error type for driver operations
#[derive(Debug, Clone)]
pub enum DriverError {
    /// OpenVR initialization error
    InitError(EVRInitError),

    /// Interface not found
    InterfaceNotFound(String),

    /// Component not found
    ComponentNotFound(String),

    /// Device not found
    DeviceNotFound(u32),

    /// Invalid parameter
    InvalidParameter(String),

    /// Operation failed
    OperationFailed(String),

    /// Hardware error
    HardwareError(String),

    /// Not implemented
    NotImplemented(String),

    /// Timeout
    Timeout,

    /// Generic error with message
    Other(String),
}

impl DriverError {
    /// Create a new init error
    pub fn init(error: EVRInitError) -> Self {
        Self::InitError(error)
    }

    /// Create a new interface not found error
    pub fn interface_not_found(name: impl Into<String>) -> Self {
        Self::InterfaceNotFound(name.into())
    }

    /// Create a new component not found error
    pub fn component_not_found(name: impl Into<String>) -> Self {
        Self::ComponentNotFound(name.into())
    }

    /// Create a new device not found error
    pub fn device_not_found(index: u32) -> Self {
        Self::DeviceNotFound(index)
    }

    /// Create a new invalid parameter error
    pub fn invalid_parameter(msg: impl Into<String>) -> Self {
        Self::InvalidParameter(msg.into())
    }

    /// Create a new operation failed error
    pub fn operation_failed(msg: impl Into<String>) -> Self {
        Self::OperationFailed(msg.into())
    }

    /// Create a new hardware error
    pub fn hardware_error(msg: impl Into<String>) -> Self {
        Self::HardwareError(msg.into())
    }

    /// Create a new not implemented error
    pub fn not_implemented(feature: impl Into<String>) -> Self {
        Self::NotImplemented(feature.into())
    }

    /// Create a new timeout error
    pub fn timeout() -> Self {
        Self::Timeout
    }

    /// Create a new generic error
    pub fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }
}

impl fmt::Display for DriverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InitError(e) => write!(f, "OpenVR initialization error: {:?}", e),
            Self::InterfaceNotFound(name) => write!(f, "Interface not found: {}", name),
            Self::ComponentNotFound(name) => write!(f, "Component not found: {}", name),
            Self::DeviceNotFound(index) => write!(f, "Device not found: {}", index),
            Self::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
            Self::OperationFailed(msg) => write!(f, "Operation failed: {}", msg),
            Self::HardwareError(msg) => write!(f, "Hardware error: {}", msg),
            Self::NotImplemented(feature) => write!(f, "Not implemented: {}", feature),
            Self::Timeout => write!(f, "Operation timed out"),
            Self::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for DriverError {}

impl From<EVRInitError> for DriverError {
    fn from(error: EVRInitError) -> Self {
        Self::InitError(error)
    }
}

/// Convert an OpenVR init error to a Result
pub fn check_init_error(error: EVRInitError) -> DriverResult<()> {
    if error == EVRInitError::None {
        Ok(())
    } else {
        Err(DriverError::InitError(error))
    }
}

/// Extension trait for converting OpenVR errors to DriverError
pub trait IntoDriverError {
    /// Convert to a DriverError
    fn into_driver_error(self) -> DriverError;
}

impl IntoDriverError for EVRInitError {
    fn into_driver_error(self) -> DriverError {
        DriverError::InitError(self)
    }
}
