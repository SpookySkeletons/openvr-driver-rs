//! Provider vtable generation
//!
//! This module handles the creation of vtables for the ServerTrackedDeviceProvider interface.

use crate::{sys, ServerTrackedDeviceProvider};
use std::ffi::{c_char, c_void};
use std::sync::{Arc, Mutex};

use super::VtableWrapper;

/// Create a vtable for a ServerTrackedDeviceProvider implementation
pub(crate) fn create_provider_vtable<T>(provider: Arc<Mutex<T>>) -> *mut c_void
where
    T: ServerTrackedDeviceProvider + 'static,
{
    use sys::root::vr::{
        EVRInitError, IServerTrackedDeviceProvider, IServerTrackedDeviceProvider__bindgen_vtable,
    };

    // Create thunk functions that forward to the Rust implementation
    unsafe extern "C" fn init_thunk<T: ServerTrackedDeviceProvider>(
        this: *mut IServerTrackedDeviceProvider,
        driver_context: *mut sys::root::vr::IVRDriverContext,
    ) -> EVRInitError {
        let wrapper =
            this as *mut VtableWrapper<IServerTrackedDeviceProvider__bindgen_vtable, Mutex<T>>;
        let provider_mutex = &(*wrapper).data;

        match provider_mutex.lock() {
            Ok(mut provider) => {
                let mut context = crate::DriverContext::from_raw(driver_context as *mut c_void);
                match provider.init(&mut context) {
                    Ok(()) => EVRInitError::None,
                    Err(e) => match e {
                        crate::DriverError::InitError(err) => err,
                        _ => EVRInitError::Unknown,
                    },
                }
            }
            Err(_) => EVRInitError::Unknown,
        }
    }

    unsafe extern "C" fn cleanup_thunk<T: ServerTrackedDeviceProvider>(
        this: *mut IServerTrackedDeviceProvider,
    ) {
        let wrapper =
            this as *mut VtableWrapper<IServerTrackedDeviceProvider__bindgen_vtable, Mutex<T>>;
        let provider_mutex = &(*wrapper).data;

        if let Ok(mut provider) = provider_mutex.lock() {
            provider.cleanup();
        }
    }

    unsafe extern "C" fn get_interface_versions_thunk<T: ServerTrackedDeviceProvider>(
        this: *mut IServerTrackedDeviceProvider,
    ) -> *const *const c_char {
        let wrapper =
            this as *mut VtableWrapper<IServerTrackedDeviceProvider__bindgen_vtable, Mutex<T>>;
        let provider_mutex = &(*wrapper).data;

        // Static storage for interface versions
        static mut VERSIONS: Vec<*const c_char> = Vec::new();
        static mut STRINGS: Vec<std::ffi::CString> = Vec::new();

        if let Ok(provider) = provider_mutex.lock() {
            let versions = provider.get_interface_versions();

            // Clear previous data
            VERSIONS.clear();
            STRINGS.clear();

            // Convert to C strings
            for version in versions {
                if let Ok(cstr) = std::ffi::CString::new(version) {
                    STRINGS.push(cstr);
                }
            }

            // Create pointer array
            for cstring in &STRINGS {
                VERSIONS.push(cstring.as_ptr());
            }
            VERSIONS.push(std::ptr::null()); // Null terminator

            VERSIONS.as_ptr()
        } else {
            std::ptr::null()
        }
    }

    unsafe extern "C" fn run_frame_thunk<T: ServerTrackedDeviceProvider>(
        this: *mut IServerTrackedDeviceProvider,
    ) {
        let wrapper =
            this as *mut VtableWrapper<IServerTrackedDeviceProvider__bindgen_vtable, Mutex<T>>;
        let provider_mutex = &(*wrapper).data;

        if let Ok(mut provider) = provider_mutex.lock() {
            provider.run_frame();
        }
    }

    unsafe extern "C" fn should_block_standby_mode_thunk<T: ServerTrackedDeviceProvider>(
        this: *mut IServerTrackedDeviceProvider,
    ) -> bool {
        let wrapper =
            this as *mut VtableWrapper<IServerTrackedDeviceProvider__bindgen_vtable, Mutex<T>>;
        let provider_mutex = &(*wrapper).data;

        if let Ok(provider) = provider_mutex.lock() {
            provider.should_block_standby_mode()
        } else {
            false
        }
    }

    unsafe extern "C" fn enter_standby_thunk<T: ServerTrackedDeviceProvider>(
        this: *mut IServerTrackedDeviceProvider,
    ) {
        let wrapper =
            this as *mut VtableWrapper<IServerTrackedDeviceProvider__bindgen_vtable, Mutex<T>>;
        let provider_mutex = &(*wrapper).data;

        if let Ok(mut provider) = provider_mutex.lock() {
            provider.enter_standby();
        }
    }

    unsafe extern "C" fn leave_standby_thunk<T: ServerTrackedDeviceProvider>(
        this: *mut IServerTrackedDeviceProvider,
    ) {
        let wrapper =
            this as *mut VtableWrapper<IServerTrackedDeviceProvider__bindgen_vtable, Mutex<T>>;
        let provider_mutex = &(*wrapper).data;

        if let Ok(mut provider) = provider_mutex.lock() {
            provider.leave_standby();
        }
    }

    // Create the vtable
    let vtable = Box::new(IServerTrackedDeviceProvider__bindgen_vtable {
        IServerTrackedDeviceProvider_Init: init_thunk::<T>,
        IServerTrackedDeviceProvider_Cleanup: cleanup_thunk::<T>,
        IServerTrackedDeviceProvider_GetInterfaceVersions: get_interface_versions_thunk::<T>,
        IServerTrackedDeviceProvider_RunFrame: run_frame_thunk::<T>,
        IServerTrackedDeviceProvider_ShouldBlockStandbyMode: should_block_standby_mode_thunk::<T>,
        IServerTrackedDeviceProvider_EnterStandby: enter_standby_thunk::<T>,
        IServerTrackedDeviceProvider_LeaveStandby: leave_standby_thunk::<T>,
    });

    let vtable_ptr = Box::into_raw(vtable);

    // Create the wrapper that contains both vtable pointer and data
    unsafe {
        let wrapper = VtableWrapper::new(vtable_ptr, provider);
        wrapper as *mut c_void
    }
}
