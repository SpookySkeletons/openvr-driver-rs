#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]
#![allow(clippy::all)]

// Include the generated bindings
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::ffi::{c_char, c_void, CStr};
use std::sync::Arc;

// Re-export key types that drivers will need
pub use root::vr::{
    DistortionCoordinates_t, DriverPose_t, ETrackedDeviceClass, ETrackingResult, EVREye,
    EVRInitError, PropertyContainerHandle_t, TrackedDeviceIndex_t, VRControllerState_t, VREvent_t,
    VRInputComponentHandle_t,
};

// Export the main interfaces
pub use root::vr::IServerTrackedDeviceProvider;
pub use root::vr::ITrackedDeviceServerDriver;
pub use root::vr::IVRDriverContext;
pub use root::vr::IVRServerDriverHost;

// Helper trait for converting Rust implementations to C++ vtables
pub trait AsVtablePtr {
    fn as_vtable_ptr(&self) -> *mut c_void;
}

// Vtable wrapper for connecting Rust implementations to C++ interfaces
#[repr(C)]
pub struct VtableWrapper<T, U> {
    pub base: T,
    pub this: Arc<U>,
}

// Interface implementation trait that all drivers must implement
pub trait InterfaceImpl {
    fn supported_versions() -> &'static [&'static CStr];
    fn get_version(version: &CStr) -> Option<Box<dyn FnOnce(&Arc<Self>) -> *mut c_void>>;
}

// Trait for inheriting from C++ interfaces
pub trait Inherits<T> {
    fn new_wrapped(this: &Arc<Self>) -> VtableWrapper<T, Self>
    where
        Self: Sized;
    fn init_fntable(this: &Arc<Self>) -> *mut c_void;
}

// Entry point that OpenVR will call
#[no_mangle]
pub unsafe extern "C" fn HmdDriverFactory(
    _interface_name: *const c_char,
    return_code: *mut i32,
) -> *mut c_void {
    // This will be implemented by the driver crate
    // For now, return null with a generic error
    if !return_code.is_null() {
        *return_code = 108; // VRInitError_Init_InterfaceNotFound
    }
    std::ptr::null_mut()
}

// Module for version-specific interface implementations
pub mod interfaces {
    use super::*;

    // This is where we'll generate version-specific traits
    // For now, just a placeholder
    pub trait IServerTrackedDeviceProvider_Interface {
        fn init(&self, driver_context: *mut c_void) -> EVRInitError;
        fn cleanup(&self);
        fn get_interface_versions(&self) -> *const *const c_char;
        fn run_frame(&self);
        fn should_block_standby_mode(&self) -> bool;
        fn enter_standby(&self);
        fn leave_standby(&self);
    }

    pub trait ITrackedDeviceServerDriver_Interface {
        fn activate(&self, device_index: u32) -> EVRInitError;
        fn deactivate(&self);
        fn enter_standby(&self);
        fn get_component(&self, component_name: *const c_char) -> *mut c_void;
        fn debug_request(
            &self,
            request: *const c_char,
            response_buffer: *mut c_char,
            response_buffer_size: u32,
        );
        fn get_pose(&self) -> DriverPose_t;
    }

    // Helper functions to create vtables for IServerTrackedDeviceProvider
    pub unsafe extern "C" fn provider_init_thunk<T: IServerTrackedDeviceProvider_Interface>(
        this: *mut root::vr::IServerTrackedDeviceProvider,
        driver_context: *mut root::vr::IVRDriverContext,
    ) -> EVRInitError {
        // The wrapper stores vtable pointer then the T
        let provider_ptr = (this as *mut u8).add(std::mem::size_of::<
            *mut root::vr::IServerTrackedDeviceProvider__bindgen_vtable,
        >()) as *mut T;
        let provider = &*provider_ptr;
        provider.init(driver_context as *mut c_void)
    }

    pub unsafe extern "C" fn provider_cleanup_thunk<T: IServerTrackedDeviceProvider_Interface>(
        this: *mut root::vr::IServerTrackedDeviceProvider,
    ) {
        let provider_ptr = (this as *mut u8).add(std::mem::size_of::<
            *mut root::vr::IServerTrackedDeviceProvider__bindgen_vtable,
        >()) as *mut T;
        let provider = &*provider_ptr;
        provider.cleanup();
    }

    pub unsafe extern "C" fn provider_get_interface_versions_thunk<
        T: IServerTrackedDeviceProvider_Interface,
    >(
        this: *mut root::vr::IServerTrackedDeviceProvider,
    ) -> *const *const c_char {
        let provider_ptr = (this as *mut u8).add(std::mem::size_of::<
            *mut root::vr::IServerTrackedDeviceProvider__bindgen_vtable,
        >()) as *mut T;
        let provider = &*provider_ptr;
        provider.get_interface_versions()
    }

    pub unsafe extern "C" fn provider_run_frame_thunk<T: IServerTrackedDeviceProvider_Interface>(
        this: *mut root::vr::IServerTrackedDeviceProvider,
    ) {
        let provider_ptr = (this as *mut u8).add(std::mem::size_of::<
            *mut root::vr::IServerTrackedDeviceProvider__bindgen_vtable,
        >()) as *mut T;
        let provider = &*provider_ptr;
        provider.run_frame();
    }

    pub unsafe extern "C" fn provider_should_block_standby_thunk<
        T: IServerTrackedDeviceProvider_Interface,
    >(
        this: *mut root::vr::IServerTrackedDeviceProvider,
    ) -> bool {
        let provider_ptr = (this as *mut u8).add(std::mem::size_of::<
            *mut root::vr::IServerTrackedDeviceProvider__bindgen_vtable,
        >()) as *mut T;
        let provider = &*provider_ptr;
        provider.should_block_standby_mode()
    }

    pub unsafe extern "C" fn provider_enter_standby_thunk<
        T: IServerTrackedDeviceProvider_Interface,
    >(
        this: *mut root::vr::IServerTrackedDeviceProvider,
    ) {
        let provider_ptr = (this as *mut u8).add(std::mem::size_of::<
            *mut root::vr::IServerTrackedDeviceProvider__bindgen_vtable,
        >()) as *mut T;
        let provider = &*provider_ptr;
        provider.enter_standby();
    }

    pub unsafe extern "C" fn provider_leave_standby_thunk<
        T: IServerTrackedDeviceProvider_Interface,
    >(
        this: *mut root::vr::IServerTrackedDeviceProvider,
    ) {
        let provider_ptr = (this as *mut u8).add(std::mem::size_of::<
            *mut root::vr::IServerTrackedDeviceProvider__bindgen_vtable,
        >()) as *mut T;
        let provider = &*provider_ptr;
        provider.leave_standby();
    }

    // Create a vtable for IServerTrackedDeviceProvider
    pub fn create_provider_vtable<T: IServerTrackedDeviceProvider_Interface + 'static>(
    ) -> *mut root::vr::IServerTrackedDeviceProvider__bindgen_vtable {
        let vtable = Box::new(root::vr::IServerTrackedDeviceProvider__bindgen_vtable {
            IServerTrackedDeviceProvider_Init: provider_init_thunk::<T>,
            IServerTrackedDeviceProvider_Cleanup: provider_cleanup_thunk::<T>,
            IServerTrackedDeviceProvider_GetInterfaceVersions:
                provider_get_interface_versions_thunk::<T>,
            IServerTrackedDeviceProvider_RunFrame: provider_run_frame_thunk::<T>,
            IServerTrackedDeviceProvider_ShouldBlockStandbyMode:
                provider_should_block_standby_thunk::<T>,
            IServerTrackedDeviceProvider_EnterStandby: provider_enter_standby_thunk::<T>,
            IServerTrackedDeviceProvider_LeaveStandby: provider_leave_standby_thunk::<T>,
        });
        Box::into_raw(vtable)
    }

    pub trait IVRDisplayComponent_Interface {
        fn is_display_on_desktop(&self) -> bool;
        fn is_display_real_display(&self) -> bool;
        fn get_recommended_render_target_size(&self, width: *mut u32, height: *mut u32);
        fn get_eye_output_viewport(
            &self,
            eye: EVREye,
            x: *mut u32,
            y: *mut u32,
            width: *mut u32,
            height: *mut u32,
        );
        fn get_projection_raw(
            &self,
            eye: EVREye,
            left: *mut f32,
            right: *mut f32,
            top: *mut f32,
            bottom: *mut f32,
        );
        fn compute_distortion(&self, eye: EVREye, u: f32, v: f32) -> DistortionCoordinates_t;
        fn get_window_bounds(&self, x: *mut i32, y: *mut i32, width: *mut u32, height: *mut u32);
    }
}

// Helper for creating vtable wrappers
#[macro_export]
macro_rules! impl_vtable_wrapper {
    ($interface:ident, $rust_impl:ty) => {
        impl $crate::AsVtablePtr for $rust_impl {
            fn as_vtable_ptr(&self) -> *mut std::ffi::c_void {
                // This will be expanded to create proper vtable
                std::ptr::null_mut()
            }
        }
    };
}

// Helper to create and box a provider with its vtable
pub fn create_provider_wrapper<T>(provider: T) -> *mut c_void
where
    T: interfaces::IServerTrackedDeviceProvider_Interface + 'static,
{
    // Create a custom structure that stores both vtable pointer and provider
    #[repr(C)]
    struct ProviderWrapper<T> {
        vtable: *mut root::vr::IServerTrackedDeviceProvider__bindgen_vtable,
        provider: T,
    }

    let vtable_ptr = interfaces::create_provider_vtable::<T>();

    let wrapper = Box::new(ProviderWrapper {
        vtable: vtable_ptr,
        provider,
    });

    Box::into_raw(wrapper) as *mut c_void
}
