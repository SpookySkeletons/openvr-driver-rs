//! Property management for OpenVR devices
//!
//! This module provides safe wrappers for reading and writing device properties
//! in the OpenVR system.

use crate::{sys, DriverError, DriverResult};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Property container handle (usually a device index)
pub type PropertyContainer = sys::root::vr::PropertyContainerHandle_t;

/// Property identifier
pub type PropertyId = sys::root::vr::ETrackedDeviceProperty;

/// Property error code
pub type PropertyError = sys::root::vr::ETrackedPropertyError;

/// A property value that can be read from or written to OpenVR
#[derive(Debug, Clone)]
pub enum PropertyValue {
    /// Boolean property
    Bool(bool),
    /// 32-bit integer property
    Int32(i32),
    /// 64-bit unsigned integer property
    Uint64(u64),
    /// 32-bit float property
    Float(f32),
    /// String property
    String(String),
    /// Matrix34 property
    Matrix34(sys::root::vr::HmdMatrix34_t),
    /// Vector3 property
    Vector3([f32; 3]),
    /// Quaternion property
    Quaternion(sys::root::vr::HmdQuaternion_t),
}

/// A property definition with its ID and value
#[derive(Debug, Clone)]
pub struct Property {
    /// The property ID
    pub id: PropertyId,
    /// The property value
    pub value: PropertyValue,
}

impl Property {
    /// Create a new property
    pub fn new(id: PropertyId, value: PropertyValue) -> Self {
        Self { id, value }
    }

    /// Create a boolean property
    pub fn bool(id: PropertyId, value: bool) -> Self {
        Self::new(id, PropertyValue::Bool(value))
    }

    /// Create an int32 property
    pub fn int32(id: PropertyId, value: i32) -> Self {
        Self::new(id, PropertyValue::Int32(value))
    }

    /// Create a uint64 property
    pub fn uint64(id: PropertyId, value: u64) -> Self {
        Self::new(id, PropertyValue::Uint64(value))
    }

    /// Create a float property
    pub fn float(id: PropertyId, value: f32) -> Self {
        Self::new(id, PropertyValue::Float(value))
    }

    /// Create a string property
    pub fn string(id: PropertyId, value: impl Into<String>) -> Self {
        Self::new(id, PropertyValue::String(value.into()))
    }
}

/// Structure for writing properties in batch
#[derive(Debug)]
pub struct PropertyWrite {
    /// The property to write
    pub property: Property,
    /// Optional error output
    pub error: Option<PropertyError>,
}

impl PropertyWrite {
    /// Create a new property write
    pub fn new(property: Property) -> Self {
        Self {
            property,
            error: None,
        }
    }

    /// Create from ID and value
    pub fn from_id_value(id: PropertyId, value: PropertyValue) -> Self {
        Self::new(Property::new(id, value))
    }
}

/// Helper functions for setting device properties
pub struct Properties;

impl Properties {
    /// Set a boolean property
    ///
    /// Note: This requires that the properties interface has been obtained
    /// through the driver context during initialization.
    pub fn set_bool(
        container: PropertyContainer,
        prop: PropertyId,
        value: bool,
    ) -> DriverResult<()> {
        // TODO: Properties need to be accessed through the driver context
        // For now, log what would be set
        eprintln!(
            "[Properties] Would set bool property {:?} = {} on container {}",
            prop, value, container
        );
        Ok(())
    }

    /// Set an int32 property
    pub fn set_int32(
        container: PropertyContainer,
        prop: PropertyId,
        value: i32,
    ) -> DriverResult<()> {
        // TODO: Properties need to be accessed through the driver context
        eprintln!(
            "[Properties] Would set int32 property {:?} = {} on container {}",
            prop, value, container
        );
        Ok(())
    }

    /// Set a uint64 property
    pub fn set_uint64(
        container: PropertyContainer,
        prop: PropertyId,
        value: u64,
    ) -> DriverResult<()> {
        // TODO: Properties need to be accessed through the driver context
        eprintln!(
            "[Properties] Would set uint64 property {:?} = {} on container {}",
            prop, value, container
        );
        Ok(())
    }

    /// Set a float property
    pub fn set_float(
        container: PropertyContainer,
        prop: PropertyId,
        value: f32,
    ) -> DriverResult<()> {
        // TODO: Properties need to be accessed through the driver context
        eprintln!(
            "[Properties] Would set float property {:?} = {} on container {}",
            prop, value, container
        );
        Ok(())
    }

    /// Set a string property
    pub fn set_string(
        container: PropertyContainer,
        prop: PropertyId,
        value: &str,
    ) -> DriverResult<()> {
        // TODO: Properties need to be accessed through the driver context
        eprintln!(
            "[Properties] Would set string property {:?} = '{}' on container {}",
            prop, value, container
        );
        Ok(())
    }

    /// Set a property from a PropertyValue
    pub fn set_property(
        container: PropertyContainer,
        prop: PropertyId,
        value: &PropertyValue,
    ) -> DriverResult<()> {
        match value {
            PropertyValue::Bool(v) => Self::set_bool(container, prop, *v),
            PropertyValue::Int32(v) => Self::set_int32(container, prop, *v),
            PropertyValue::Uint64(v) => Self::set_uint64(container, prop, *v),
            PropertyValue::Float(v) => Self::set_float(container, prop, *v),
            PropertyValue::String(v) => Self::set_string(container, prop, v),
            PropertyValue::Matrix34(_)
            | PropertyValue::Vector3(_)
            | PropertyValue::Quaternion(_) => {
                // These types need special handling
                Err(DriverError::not_implemented(
                    "Matrix34/Vector3/Quaternion properties",
                ))
            }
        }
    }

    /// Write multiple properties in batch
    ///
    /// This is the preferred way to set multiple properties at once,
    /// as it's more efficient than setting them individually.
    pub fn write_batch(
        container: PropertyContainer,
        writes: &mut [PropertyWrite],
    ) -> DriverResult<()> {
        // For now, we'll implement this as individual writes
        // TODO: Implement actual batch writing using PropertyWrite_t
        for write in writes {
            match Self::set_property(container, write.property.id, &write.property.value) {
                Ok(()) => write.error = Some(PropertyError::TrackedProp_Success),
                Err(_) => write.error = Some(PropertyError::TrackedProp_UnknownProperty),
            }
        }
        Ok(())
    }
}

/// Helper functions for common HMD properties
pub fn set_hmd_properties(
    device_index: u32,
    model_number: &str,
    serial_number: &str,
    display_frequency: f32,
    ipd: f32,
) -> DriverResult<()> {
    // TODO: These property enum values need to be verified against the actual bindings
    // For now, we'll just log what would be set
    eprintln!(
        "[Properties] Setting HMD properties for device {}:",
        device_index
    );
    eprintln!("  Model: {}", model_number);
    eprintln!("  Serial: {}", serial_number);
    eprintln!("  Display Frequency: {} Hz", display_frequency);
    eprintln!("  IPD: {} meters", ipd);
    eprintln!("  Is On Desktop: true");
    eprintln!("  Universe ID: 2");

    Ok(())
}

/// Helper functions for common controller properties
pub fn set_controller_properties(
    device_index: u32,
    model_number: &str,
    serial_number: &str,
    hand: ControllerHand,
) -> DriverResult<()> {
    // TODO: These property enum values need to be verified against the actual bindings
    eprintln!(
        "[Properties] Setting controller properties for device {}:",
        device_index
    );
    eprintln!("  Model: {}", model_number);
    eprintln!("  Serial: {}", serial_number);
    eprintln!("  Hand: {:?}", hand);

    Ok(())
}

/// Controller hand designation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControllerHand {
    Left,
    Right,
}
