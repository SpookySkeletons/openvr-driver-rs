//! Driver context and host interface management
//!
//! This module provides safe wrappers around OpenVR's driver context
//! and host interfaces, handling the low-level FFI details.

use crate::{properties, sys, DriverError, DriverResult, TrackedDeviceServerDriver};
use std::ffi::{c_void, CStr, CString};
use std::sync::Arc;

/// Driver context for accessing OpenVR interfaces
///
/// This struct provides access to OpenVR's driver interfaces and
/// allows registering devices with the system.
pub struct DriverContext {
    /// Raw pointer to IVRDriverContext
    context: *mut sys::root::vr::IVRDriverContext,
    /// Driver host interface
    host: Option<DriverHost>,
    /// Properties interface
    properties: Option<*mut sys::root::vr::IVRProperties>,
    /// Driver input interface
    driver_input: Option<*mut sys::root::vr::IVRDriverInput>,
}

unsafe impl Send for DriverContext {}
unsafe impl Sync for DriverContext {}

impl DriverContext {
    /// Create a new driver context from a raw pointer
    ///
    /// # Safety
    /// The provided pointer must be a valid IVRDriverContext pointer
    /// that remains valid for the lifetime of this DriverContext.
    pub unsafe fn from_raw(context: *mut c_void) -> Self {
        let context = context as *mut sys::root::vr::IVRDriverContext;

        let mut driver_context = Self {
            context,
            host: None,
            properties: None,
            driver_input: None,
        };

        // Try to get the host interface
        if let Ok(host) = driver_context.get_server_driver_host() {
            driver_context.host = Some(host);
        }

        // Try to get properties interface
        if let Ok(props) = driver_context.get_properties_interface() {
            driver_context.properties = Some(props);
            // Set the global properties interface for property operations
            unsafe {
                properties::set_properties_interface(props);
            }
        }

        // Try to get driver input interface
        if let Ok(input) = driver_context.get_driver_input_interface() {
            driver_context.driver_input = Some(input);
        }

        driver_context
    }

    /// Get the server driver host interface
    fn get_server_driver_host(&self) -> DriverResult<DriverHost> {
        unsafe {
            let interface_name = CString::new("IVRServerDriverHost_006").unwrap();
            let mut error = sys::root::vr::EVRInitError::None;

            let vtable = (*self.context).vtable_;
            let get_interface = (*vtable).IVRDriverContext_GetGenericInterface;

            let host_ptr = get_interface(self.context, interface_name.as_ptr(), &mut error);

            if error != sys::root::vr::EVRInitError::None {
                return Err(DriverError::InitError(error));
            }

            if host_ptr.is_null() {
                return Err(DriverError::InterfaceNotFound(
                    "IVRServerDriverHost".to_string(),
                ));
            }

            Ok(DriverHost::from_raw(
                host_ptr as *mut sys::root::vr::IVRServerDriverHost,
            ))
        }
    }

    /// Get the properties interface
    fn get_properties_interface(&self) -> DriverResult<*mut sys::root::vr::IVRProperties> {
        unsafe {
            let interface_name = CString::new("IVRProperties_001").unwrap();
            let mut error = sys::root::vr::EVRInitError::None;

            let vtable = (*self.context).vtable_;
            let get_interface = (*vtable).IVRDriverContext_GetGenericInterface;

            let props_ptr = get_interface(self.context, interface_name.as_ptr(), &mut error);

            if error != sys::root::vr::EVRInitError::None {
                return Err(DriverError::InitError(error));
            }

            if props_ptr.is_null() {
                return Err(DriverError::InterfaceNotFound("IVRProperties".to_string()));
            }

            Ok(props_ptr as *mut sys::root::vr::IVRProperties)
        }
    }

    /// Get the driver input interface
    fn get_driver_input_interface(&self) -> DriverResult<*mut sys::root::vr::IVRDriverInput> {
        unsafe {
            let interface_name = CString::new("IVRDriverInput_003").unwrap();
            let mut error = sys::root::vr::EVRInitError::None;

            let vtable = (*self.context).vtable_;
            let get_interface = (*vtable).IVRDriverContext_GetGenericInterface;

            let input_ptr = get_interface(self.context, interface_name.as_ptr(), &mut error);

            if error != sys::root::vr::EVRInitError::None {
                return Err(DriverError::InitError(error));
            }

            if input_ptr.is_null() {
                return Err(DriverError::InterfaceNotFound("IVRDriverInput".to_string()));
            }

            Ok(input_ptr as *mut sys::root::vr::IVRDriverInput)
        }
    }

    /// Register a device with OpenVR
    ///
    /// This method registers a tracked device with the OpenVR system.
    /// The device will be assigned an index when activated.
    ///
    /// # Arguments
    /// * `device` - The device to register
    ///
    /// # Returns
    /// * `Ok(())` if registration succeeded
    /// * `Err(DriverError)` if registration failed
    pub fn register_device(
        &mut self,
        device: Arc<dyn TrackedDeviceServerDriver>,
    ) -> DriverResult<()> {
        let host = self
            .host
            .as_ref()
            .ok_or_else(|| DriverError::InterfaceNotFound("IVRServerDriverHost".to_string()))?;

        host.tracked_device_added(device)
    }

    /// Get the driver host interface
    ///
    /// The host interface provides methods for registering devices
    /// and sending events to the OpenVR runtime.
    pub fn host(&self) -> Option<&DriverHost> {
        self.host.as_ref()
    }

    /// Get a mutable reference to the driver host interface
    pub fn host_mut(&mut self) -> Option<&mut DriverHost> {
        self.host.as_mut()
    }

    /// Get the raw context pointer
    ///
    /// # Safety
    /// The returned pointer should not be stored beyond the lifetime
    /// of this DriverContext.
    pub unsafe fn raw_context(&self) -> *mut sys::root::vr::IVRDriverContext {
        self.context
    }

    /// Get the raw properties interface pointer
    ///
    /// # Safety
    /// The returned pointer should not be stored beyond the lifetime
    /// of this DriverContext.
    pub unsafe fn raw_properties(&self) -> Option<*mut sys::root::vr::IVRProperties> {
        self.properties
    }

    /// Get the raw driver input interface pointer
    ///
    /// # Safety
    /// The returned pointer should not be stored beyond the lifetime
    /// of this DriverContext.
    pub unsafe fn raw_driver_input(&self) -> Option<*mut sys::root::vr::IVRDriverInput> {
        self.driver_input
    }
}

/// Driver host interface for interacting with OpenVR
///
/// This struct provides methods for registering devices and sending
/// events to the OpenVR runtime.
pub struct DriverHost {
    /// Raw pointer to IVRServerDriverHost
    host: *mut sys::root::vr::IVRServerDriverHost,
}

unsafe impl Send for DriverHost {}
unsafe impl Sync for DriverHost {}

impl DriverHost {
    /// Create a new driver host from a raw pointer
    ///
    /// # Safety
    /// The provided pointer must be a valid IVRServerDriverHost pointer.
    unsafe fn from_raw(host: *mut sys::root::vr::IVRServerDriverHost) -> Self {
        Self { host }
    }

    /// Register a tracked device with OpenVR
    ///
    /// # Arguments
    /// * `device` - The device to register
    pub fn tracked_device_added(
        &self,
        device: Arc<dyn TrackedDeviceServerDriver>,
    ) -> DriverResult<()> {
        unsafe {
            // Get the serial number from the device
            let serial_string = device.get_serial_number();
            eprintln!(
                "[DriverHost] Registering device with serial: {}",
                serial_string
            );
            let serial = CString::new(serial_string.as_str())
                .map_err(|_| DriverError::invalid_parameter("Serial number contains null byte"))?;

            // Create the device vtable wrapper
            let device_vtable = crate::vtables::create_device_vtable(device);
            eprintln!("[DriverHost] Created device vtable at {:?}", device_vtable);

            let vtable = (*self.host).vtable_;
            let tracked_device_added = (*vtable).IVRServerDriverHost_TrackedDeviceAdded;

            eprintln!("[DriverHost] Calling TrackedDeviceAdded with:");
            eprintln!("  Serial: {}", serial_string);
            eprintln!("  Class: HMD");
            eprintln!("  Device vtable: {:?}", device_vtable);

            let result = tracked_device_added(
                self.host,
                serial.as_ptr(),
                sys::root::vr::ETrackedDeviceClass::TrackedDeviceClass_HMD,
                device_vtable as *mut sys::root::vr::ITrackedDeviceServerDriver,
            );

            if result {
                eprintln!("[DriverHost] TrackedDeviceAdded returned success");
                Ok(())
            } else {
                eprintln!("[DriverHost] TrackedDeviceAdded returned failure");
                Err(DriverError::OperationFailed(
                    "Failed to register device".to_string(),
                ))
            }
        }
    }

    /// Poll for next event
    ///
    /// # Arguments
    /// * `event` - Buffer to receive the event
    ///
    /// # Returns
    /// * `true` if an event was retrieved
    /// * `false` if no events are pending
    pub fn poll_next_event(&self, event: &mut sys::root::vr::VREvent_t) -> bool {
        unsafe {
            let vtable = (*self.host).vtable_;
            let poll_next_event = (*vtable).IVRServerDriverHost_PollNextEvent;

            let event_size = std::mem::size_of::<sys::root::vr::VREvent_t>() as u32;
            poll_next_event(self.host, event, event_size)
        }
    }

    /// Get the raw host pointer
    ///
    /// # Safety
    /// The returned pointer should not be stored beyond the lifetime
    /// of this DriverHost.
    pub unsafe fn raw_host(&self) -> *mut sys::root::vr::IVRServerDriverHost {
        self.host
    }
}
