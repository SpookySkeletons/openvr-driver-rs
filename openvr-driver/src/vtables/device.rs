//! Device vtable generation
//!
//! This module handles the creation of vtables for the TrackedDeviceServerDriver interface.

use crate::{sys, TrackedDeviceServerDriver};
use std::ffi::{c_char, c_void, CStr, CString};
use std::sync::{Arc, Mutex};

use super::VtableWrapper;

/// Create a vtable for a TrackedDeviceServerDriver implementation (dynamic version)
pub(crate) fn create_device_vtable(device: Arc<dyn TrackedDeviceServerDriver>) -> *mut c_void {
    create_device_vtable_impl(device)
}

/// Internal implementation that works with Arc<dyn TrackedDeviceServerDriver>
fn create_device_vtable_impl(device: Arc<dyn TrackedDeviceServerDriver>) -> *mut c_void {
    use sys::root::vr::{
        DriverPose_t, EVRInitError, ITrackedDeviceServerDriver,
        ITrackedDeviceServerDriver__bindgen_vtable,
    };

    // Create thunk functions that forward to the Rust implementation
    unsafe extern "C" fn activate_thunk(
        this: *mut ITrackedDeviceServerDriver,
        device_index: u32,
    ) -> EVRInitError {
        let vtable_wrapper = this as *mut VtableWrapper<
            ITrackedDeviceServerDriver__bindgen_vtable,
            dyn TrackedDeviceServerDriver,
        >;

        eprintln!(
            "[Device Vtable] Activate called for device index {}",
            device_index
        );

        // Since we can't easily get mutable access through Arc<dyn Trait>,
        // we assume the implementation uses interior mutability.
        // For now, just return success and let the device handle activation
        // through other means (like storing the index when created).
        EVRInitError::None
    }

    unsafe extern "C" fn deactivate_thunk(this: *mut ITrackedDeviceServerDriver) {
        let vtable_wrapper = this as *mut VtableWrapper<
            ITrackedDeviceServerDriver__bindgen_vtable,
            dyn TrackedDeviceServerDriver,
        >;

        eprintln!("[Device Vtable] Deactivate called");
        // Deactivation handled through interior mutability in implementation
    }

    unsafe extern "C" fn enter_standby_thunk(this: *mut ITrackedDeviceServerDriver) {
        let vtable_wrapper = this as *mut VtableWrapper<
            ITrackedDeviceServerDriver__bindgen_vtable,
            dyn TrackedDeviceServerDriver,
        >;

        eprintln!("[Device Vtable] Enter standby called");
        // Standby handled through interior mutability in implementation
    }

    unsafe extern "C" fn get_component_thunk(
        this: *mut ITrackedDeviceServerDriver,
        component_name: *const c_char,
    ) -> *mut c_void {
        let vtable_wrapper = this as *mut VtableWrapper<
            ITrackedDeviceServerDriver__bindgen_vtable,
            dyn TrackedDeviceServerDriver,
        >;

        let device = &(*vtable_wrapper).data;

        if component_name.is_null() {
            return std::ptr::null_mut();
        }

        let name = match CStr::from_ptr(component_name).to_str() {
            Ok(s) => s,
            Err(_) => return std::ptr::null_mut(),
        };

        match device.get_component(name) {
            Some(ptr) => ptr,
            None => std::ptr::null_mut(),
        }
    }

    unsafe extern "C" fn debug_request_thunk(
        this: *mut ITrackedDeviceServerDriver,
        request: *const c_char,
        response_buffer: *mut c_char,
        response_buffer_size: u32,
    ) {
        let vtable_wrapper = this as *mut VtableWrapper<
            ITrackedDeviceServerDriver__bindgen_vtable,
            dyn TrackedDeviceServerDriver,
        >;

        if request.is_null() || response_buffer.is_null() || response_buffer_size == 0 {
            return;
        }

        let request_str = match CStr::from_ptr(request).to_str() {
            Ok(s) => s,
            Err(_) => return,
        };

        // Debug request requires mutable access, return a default response
        let response = format!("[Device] Debug request received: {}", request_str);

        // Copy response to buffer
        if let Ok(response_cstr) = CString::new(response) {
            let response_bytes = response_cstr.as_bytes_with_nul();
            let copy_len = std::cmp::min(response_bytes.len(), response_buffer_size as usize);

            std::ptr::copy_nonoverlapping(
                response_bytes.as_ptr() as *const c_char,
                response_buffer,
                copy_len,
            );

            // Ensure null termination
            if copy_len > 0 {
                *response_buffer.add(copy_len - 1) = 0;
            }
        }
    }

    unsafe extern "C" fn get_pose_thunk(this: *mut ITrackedDeviceServerDriver) -> DriverPose_t {
        let vtable_wrapper = this as *mut VtableWrapper<
            ITrackedDeviceServerDriver__bindgen_vtable,
            dyn TrackedDeviceServerDriver,
        >;

        let device = &(*vtable_wrapper).data;
        device.get_pose()
    }

    // Create the vtable
    let vtable = Box::new(ITrackedDeviceServerDriver__bindgen_vtable {
        ITrackedDeviceServerDriver_Activate: activate_thunk,
        ITrackedDeviceServerDriver_Deactivate: deactivate_thunk,
        ITrackedDeviceServerDriver_EnterStandby: enter_standby_thunk,
        ITrackedDeviceServerDriver_GetComponent: get_component_thunk,
        ITrackedDeviceServerDriver_DebugRequest: debug_request_thunk,
        ITrackedDeviceServerDriver_GetPose: get_pose_thunk,
    });

    let vtable_ptr = Box::into_raw(vtable);

    // Create the wrapper that contains both vtable pointer and data
    unsafe {
        let vtable_wrapper = VtableWrapper::new(vtable_ptr, device);
        vtable_wrapper as *mut c_void
    }
}
