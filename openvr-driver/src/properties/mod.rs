//! Property management for OpenVR devices
//!
//! This module provides safe wrappers for reading and writing device properties
//! in the OpenVR system.

use crate::{sys, DriverError, DriverResult};
use std::ffi::{CStr, CString};
use std::mem;
use std::os::raw::{c_char, c_void};
use std::ptr;

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

/// Global storage for the properties interface pointer
/// This is set during driver initialization through the context
static mut PROPERTIES_INTERFACE: Option<*mut sys::root::vr::IVRProperties> = None;

/// Set the global properties interface pointer
///
/// # Safety
/// This should only be called once during driver initialization
pub unsafe fn set_properties_interface(properties: *mut sys::root::vr::IVRProperties) {
    PROPERTIES_INTERFACE = Some(properties);
}

/// Get a property container handle for a device index
///
/// This converts a device index to a property container handle that can be
/// used with the property system.
pub fn get_property_container(device_index: u32) -> PropertyContainer {
    unsafe {
        eprintln!(
            "[Properties] Getting container for device index {}",
            device_index
        );

        if let Some(properties_ptr) = PROPERTIES_INTERFACE {
            eprintln!(
                "[Properties] Properties interface exists: {:?}",
                properties_ptr
            );

            if !properties_ptr.is_null() {
                let vtable = (*properties_ptr).vtable_;
                let get_container = (*vtable).IVRProperties_TrackedDeviceToPropertyContainer;
                let container = get_container(properties_ptr, device_index);

                eprintln!(
                    "[Properties] Got container {} for device index {}",
                    container, device_index
                );
                return container;
            } else {
                eprintln!("[Properties] ERROR: Properties interface is null!");
            }
        } else {
            eprintln!("[Properties] ERROR: Properties interface not initialized!");
        }

        // Return the device index as container if we can't get the real one
        eprintln!(
            "[Properties] WARNING: Using device index {} as fallback container",
            device_index
        );
        device_index as PropertyContainer
    }
}

/// Helper functions for setting device properties
pub struct Properties;

impl Properties {
    /// Write multiple properties in a batch (fixes Error 496)
    ///
    /// This is the primary method for setting device properties in OpenVR.
    pub fn write_property_batch(
        container: PropertyContainer,
        properties: &[(PropertyId, PropertyValue)],
    ) -> DriverResult<()> {
        unsafe {
            eprintln!(
                "[Properties] write_property_batch called for container {}",
                container
            );

            let properties_ptr = PROPERTIES_INTERFACE.ok_or_else(|| {
                eprintln!(
                    "[Properties] ERROR: IVRProperties not initialized in write_property_batch"
                );
                DriverError::interface_not_found("IVRProperties not initialized")
            })?;

            if properties_ptr.is_null() {
                eprintln!("[Properties] ERROR: IVRProperties is null in write_property_batch");
                return Err(DriverError::interface_not_found("IVRProperties is null"));
            }

            eprintln!(
                "[Properties] Properties interface valid, preparing {} properties",
                properties.len()
            );

            // Create PropertyWrite_t array
            let mut writes: Vec<sys::root::vr::PropertyWrite_t> =
                Vec::with_capacity(properties.len());

            // Store data that needs to live through the FFI call
            let mut string_data: Vec<CString> = Vec::new();

            for (prop_id, value) in properties {
                let mut write = sys::root::vr::PropertyWrite_t {
                    prop: *prop_id,
                    writeType: sys::root::vr::EPropertyWriteType::PropertyWrite_Set,
                    eSetError: sys::root::vr::ETrackedPropertyError::TrackedProp_Success,
                    pvBuffer: ptr::null_mut(),
                    unBufferSize: 0,
                    unTag: 0,
                    eError: sys::root::vr::ETrackedPropertyError::TrackedProp_Success,
                };

                match value {
                    PropertyValue::Bool(v) => {
                        write.pvBuffer = v as *const bool as *mut c_void;
                        write.unBufferSize = mem::size_of::<bool>() as u32;
                        write.unTag = sys::root::vr::k_unBoolPropertyTag;
                    }
                    PropertyValue::Float(v) => {
                        write.pvBuffer = v as *const f32 as *mut c_void;
                        write.unBufferSize = mem::size_of::<f32>() as u32;
                        write.unTag = sys::root::vr::k_unFloatPropertyTag;
                    }
                    PropertyValue::Int32(v) => {
                        write.pvBuffer = v as *const i32 as *mut c_void;
                        write.unBufferSize = mem::size_of::<i32>() as u32;
                        write.unTag = sys::root::vr::k_unInt32PropertyTag;
                    }
                    PropertyValue::Uint64(v) => {
                        write.pvBuffer = v as *const u64 as *mut c_void;
                        write.unBufferSize = mem::size_of::<u64>() as u32;
                        write.unTag = sys::root::vr::k_unUint64PropertyTag;
                    }
                    PropertyValue::String(s) => {
                        let cstring = CString::new(s.as_str()).map_err(|_| {
                            DriverError::invalid_parameter("String contains null byte")
                        })?;
                        write.pvBuffer = cstring.as_ptr() as *mut c_void;
                        write.unBufferSize = (cstring.as_bytes_with_nul().len()) as u32;
                        write.unTag = sys::root::vr::k_unStringPropertyTag;
                        string_data.push(cstring);
                    }
                    _ => {
                        eprintln!("[Properties] Unsupported property type, skipping");
                        continue;
                    }
                }

                writes.push(write);
            }

            if writes.is_empty() {
                eprintln!("[Properties] WARNING: No properties to write after processing");
                return Ok(());
            }

            eprintln!(
                "[Properties] Calling WritePropertyBatch with {} properties",
                writes.len()
            );

            // Call WritePropertyBatch
            let vtable = (*properties_ptr).vtable_;
            let write_batch = (*vtable).IVRProperties_WritePropertyBatch;

            let error = write_batch(
                properties_ptr,
                container,
                writes.as_mut_ptr(),
                writes.len() as u32,
            );

            if error == sys::root::vr::ETrackedPropertyError::TrackedProp_Success {
                eprintln!(
                    "[Properties] Successfully wrote {} properties",
                    writes.len()
                );
                Ok(())
            } else {
                eprintln!("[Properties] Failed to write properties: {:?}", error);
                Err(DriverError::operation_failed(format!(
                    "WritePropertyBatch failed: {:?}",
                    error
                )))
            }
        }
    }

    /// Set a boolean property
    ///
    /// Note: This requires that the properties interface has been obtained
    /// through the driver context during initialization.
    pub fn set_bool(
        container: PropertyContainer,
        prop: PropertyId,
        value: bool,
    ) -> DriverResult<()> {
        Properties::write_property_batch(container, &[(prop, PropertyValue::Bool(value))])
    }

    /// Set an int32 property
    pub fn set_int32(
        container: PropertyContainer,
        prop: PropertyId,
        value: i32,
    ) -> DriverResult<()> {
        Properties::write_property_batch(container, &[(prop, PropertyValue::Int32(value))])
    }

    /// Set a uint64 property
    pub fn set_uint64(
        container: PropertyContainer,
        prop: PropertyId,
        value: u64,
    ) -> DriverResult<()> {
        Properties::write_property_batch(container, &[(prop, PropertyValue::Uint64(value))])
    }

    /// Set a float property
    pub fn set_float(
        container: PropertyContainer,
        prop: PropertyId,
        value: f32,
    ) -> DriverResult<()> {
        Properties::write_property_batch(container, &[(prop, PropertyValue::Float(value))])
    }

    /// Set a string property
    pub fn set_string(
        container: PropertyContainer,
        prop: PropertyId,
        value: &str,
    ) -> DriverResult<()> {
        Properties::write_property_batch(
            container,
            &[(prop, PropertyValue::String(value.to_string()))],
        )
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
    // Get the proper property container for this device
    let container = get_property_container(device_index);

    eprintln!(
        "[Properties] Setting HMD properties for device {} with container {}",
        device_index, container
    );

    // Check if container is valid (0 means invalid)
    if container == 0 {
        eprintln!(
            "[Properties] ERROR: Invalid property container (0) for device {}",
            device_index
        );
        return Err(DriverError::operation_failed("Invalid property container"));
    }

    // Create batch of properties to write - matching C++ implementation exactly
    let properties = vec![
        (
            sys::root::vr::ETrackedDeviceProperty::Prop_ModelNumber_String,
            PropertyValue::String(model_number.to_string()),
        ),
        (
            sys::root::vr::ETrackedDeviceProperty::Prop_SerialNumber_String,
            PropertyValue::String(serial_number.to_string()),
        ),
        // IMPORTANT: C++ sets this to 0.0f, not the actual frequency!
        (
            sys::root::vr::ETrackedDeviceProperty::Prop_DisplayFrequency_Float,
            PropertyValue::Float(0.0),
        ),
        (
            sys::root::vr::ETrackedDeviceProperty::Prop_UserIpdMeters_Float,
            PropertyValue::Float(ipd),
        ),
        // The distance from user's eyes to display in meters - for reprojection
        (
            sys::root::vr::ETrackedDeviceProperty::Prop_UserHeadToEyeDepthMeters_Float,
            PropertyValue::Float(0.0),
        ),
        // Time from compositor submit to photons on screen
        (
            sys::root::vr::ETrackedDeviceProperty::Prop_SecondsFromVsyncToPhotons_Float,
            PropertyValue::Float(0.11),
        ),
        // IMPORTANT: C++ sets this to false to avoid "not fullscreen" warnings
        (
            sys::root::vr::ETrackedDeviceProperty::Prop_IsOnDesktop_Bool,
            PropertyValue::Bool(false),
        ),
        // Enable display debug mode
        (
            sys::root::vr::ETrackedDeviceProperty::Prop_DisplayDebugMode_Bool,
            PropertyValue::Bool(true),
        ),
        (
            sys::root::vr::ETrackedDeviceProperty::Prop_CurrentUniverseId_Uint64,
            PropertyValue::Uint64(2),
        ),
        // Add manufacturer and tracking system names
        (
            sys::root::vr::ETrackedDeviceProperty::Prop_ManufacturerName_String,
            PropertyValue::String("Simple VR".to_string()),
        ),
        (
            sys::root::vr::ETrackedDeviceProperty::Prop_TrackingSystemName_String,
            PropertyValue::String("simplehmd".to_string()),
        ),
    ];

    // Write all properties in a single batch
    Properties::write_property_batch(container, &properties)
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
