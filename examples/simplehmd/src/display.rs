use openvr_driver_bindings::{
    interfaces::IVRDisplayComponent_Interface,
    root::vr::{DistortionCoordinates_t, EVREye},
};
use std::sync::Arc;

use crate::DisplayConfiguration;

// Display component for the HMD
pub struct DisplayComponent {
    config: DisplayConfiguration,
}

impl DisplayComponent {
    pub fn new(config: DisplayConfiguration) -> Arc<Self> {
        Arc::new(Self { config })
    }
}

// Wrapper to implement the trait
pub struct DisplayComponentWrapper(pub Arc<DisplayComponent>);

impl IVRDisplayComponent_Interface for DisplayComponentWrapper {
    fn is_display_on_desktop(&self) -> bool {
        // CRITICAL: For extended mode (non-direct mode), this must be true
        // This tells SteamVR the display is part of the desktop
        true
    }

    fn is_display_real_display(&self) -> bool {
        // CRITICAL: Must be true for SteamVR to find the display
        // false = virtual display that doesn't exist
        // true = real display output SteamVR can render to
        true
    }

    fn get_recommended_render_target_size(&self, width: *mut u32, height: *mut u32) {
        unsafe {
            if !width.is_null() {
                *width = self.0.config.render_width as u32;
            }
            if !height.is_null() {
                *height = self.0.config.render_height as u32;
            }
        }
    }

    fn get_eye_output_viewport(
        &self,
        eye: EVREye,
        x: *mut u32,
        y: *mut u32,
        width: *mut u32,
        height: *mut u32,
    ) {
        unsafe {
            if !y.is_null() {
                *y = 0;
            }

            // Each eye gets half the window width
            let eye_width = self.0.config.window_width / 2;
            if !width.is_null() {
                *width = eye_width as u32;
            }

            // Full height for each eye
            if !height.is_null() {
                *height = self.0.config.window_height as u32;
            }

            // Left eye on the left, right eye on the right
            if !x.is_null() {
                *x = match eye {
                    EVREye::Eye_Left => 0,
                    EVREye::Eye_Right => eye_width as u32,
                };
            }
        }
    }

    fn get_projection_raw(
        &self,
        _eye: EVREye,
        left: *mut f32,
        right: *mut f32,
        top: *mut f32,
        bottom: *mut f32,
    ) {
        // Simple symmetric projection for now
        unsafe {
            if !left.is_null() {
                *left = -1.0;
            }
            if !right.is_null() {
                *right = 1.0;
            }
            if !top.is_null() {
                *top = -1.0;
            }
            if !bottom.is_null() {
                *bottom = 1.0;
            }
        }
    }

    fn compute_distortion(&self, _eye: EVREye, u: f32, v: f32) -> DistortionCoordinates_t {
        // No distortion - pass through coordinates
        DistortionCoordinates_t {
            rfRed: [u, v],
            rfGreen: [u, v],
            rfBlue: [u, v],
        }
    }

    fn get_window_bounds(&self, x: *mut i32, y: *mut i32, width: *mut u32, height: *mut u32) {
        unsafe {
            // IMPORTANT: These coordinates must match an actual display on the system
            // For testing, use primary monitor at 0,0
            // In production, you'd detect an actual secondary display
            if !x.is_null() {
                *x = 0; // Primary monitor X position
            }
            if !y.is_null() {
                *y = 0; // Primary monitor Y position
            }
            if !width.is_null() {
                *width = self.0.config.window_width as u32;
            }
            if !height.is_null() {
                *height = self.0.config.window_height as u32;
            }
        }
    }
}

// Helper to create display component vtable
pub fn create_display_vtable<T: IVRDisplayComponent_Interface + 'static>(
    display: T,
) -> *mut openvr_driver_bindings::root::vr::IVRDisplayComponent {
    use openvr_driver_bindings::root::vr::{
        IVRDisplayComponent, IVRDisplayComponent__bindgen_vtable,
    };

    // Create thunk functions for the vtable
    unsafe extern "C" fn is_display_on_desktop_thunk<T: IVRDisplayComponent_Interface>(
        this: *mut IVRDisplayComponent,
    ) -> bool {
        let display_ptr = (this as *mut u8)
            .add(std::mem::size_of::<*mut IVRDisplayComponent__bindgen_vtable>())
            as *mut T;
        let display = &*display_ptr;
        display.is_display_on_desktop()
    }

    unsafe extern "C" fn is_display_real_display_thunk<T: IVRDisplayComponent_Interface>(
        this: *mut IVRDisplayComponent,
    ) -> bool {
        let display_ptr = (this as *mut u8)
            .add(std::mem::size_of::<*mut IVRDisplayComponent__bindgen_vtable>())
            as *mut T;
        let display = &*display_ptr;
        display.is_display_real_display()
    }

    unsafe extern "C" fn get_recommended_render_target_size_thunk<
        T: IVRDisplayComponent_Interface,
    >(
        this: *mut IVRDisplayComponent,
        width: *mut u32,
        height: *mut u32,
    ) {
        let display_ptr = (this as *mut u8)
            .add(std::mem::size_of::<*mut IVRDisplayComponent__bindgen_vtable>())
            as *mut T;
        let display = &*display_ptr;
        display.get_recommended_render_target_size(width, height);
    }

    unsafe extern "C" fn get_eye_output_viewport_thunk<T: IVRDisplayComponent_Interface>(
        this: *mut IVRDisplayComponent,
        eye: EVREye,
        x: *mut u32,
        y: *mut u32,
        width: *mut u32,
        height: *mut u32,
    ) {
        let display_ptr = (this as *mut u8)
            .add(std::mem::size_of::<*mut IVRDisplayComponent__bindgen_vtable>())
            as *mut T;
        let display = &*display_ptr;
        display.get_eye_output_viewport(eye, x, y, width, height);
    }

    unsafe extern "C" fn get_projection_raw_thunk<T: IVRDisplayComponent_Interface>(
        this: *mut IVRDisplayComponent,
        eye: EVREye,
        left: *mut f32,
        right: *mut f32,
        top: *mut f32,
        bottom: *mut f32,
    ) {
        let display_ptr = (this as *mut u8)
            .add(std::mem::size_of::<*mut IVRDisplayComponent__bindgen_vtable>())
            as *mut T;
        let display = &*display_ptr;
        display.get_projection_raw(eye, left, right, top, bottom);
    }

    unsafe extern "C" fn compute_distortion_thunk<T: IVRDisplayComponent_Interface>(
        this: *mut IVRDisplayComponent,
        eye: EVREye,
        u: f32,
        v: f32,
    ) -> DistortionCoordinates_t {
        let display_ptr = (this as *mut u8)
            .add(std::mem::size_of::<*mut IVRDisplayComponent__bindgen_vtable>())
            as *mut T;
        let display = &*display_ptr;
        display.compute_distortion(eye, u, v)
    }

    unsafe extern "C" fn get_window_bounds_thunk<T: IVRDisplayComponent_Interface>(
        this: *mut IVRDisplayComponent,
        x: *mut i32,
        y: *mut i32,
        width: *mut u32,
        height: *mut u32,
    ) {
        let display_ptr = (this as *mut u8)
            .add(std::mem::size_of::<*mut IVRDisplayComponent__bindgen_vtable>())
            as *mut T;
        let display = &*display_ptr;
        display.get_window_bounds(x, y, width, height);
    }

    // ComputeInverseDistortion is the inverse of ComputeDistortion
    // For simplicity, we'll just pass through coordinates since we have no distortion
    unsafe extern "C" fn compute_inverse_distortion_thunk<T: IVRDisplayComponent_Interface>(
        _this: *mut IVRDisplayComponent,
        pc_out: *mut openvr_driver_bindings::root::vr::HmdVector2_t,
        _eye: EVREye,
        _u: u32,
        _v: f32,
        _w: f32,
    ) -> bool {
        if !pc_out.is_null() {
            *pc_out = openvr_driver_bindings::root::vr::HmdVector2_t { v: [_v, _w] };
            true
        } else {
            false
        }
    }

    // Create the vtable
    let vtable = Box::new(IVRDisplayComponent__bindgen_vtable {
        IVRDisplayComponent_IsDisplayOnDesktop: is_display_on_desktop_thunk::<T>,
        IVRDisplayComponent_IsDisplayRealDisplay: is_display_real_display_thunk::<T>,
        IVRDisplayComponent_GetRecommendedRenderTargetSize:
            get_recommended_render_target_size_thunk::<T>,
        IVRDisplayComponent_GetEyeOutputViewport: get_eye_output_viewport_thunk::<T>,
        IVRDisplayComponent_GetProjectionRaw: get_projection_raw_thunk::<T>,
        IVRDisplayComponent_ComputeDistortion: compute_distortion_thunk::<T>,
        IVRDisplayComponent_ComputeInverseDistortion: compute_inverse_distortion_thunk::<T>,
        IVRDisplayComponent_GetWindowBounds: get_window_bounds_thunk::<T>,
    });
    let vtable_ptr = Box::into_raw(vtable);

    // Create a custom structure that stores both vtable pointer and display
    #[repr(C)]
    struct DisplayWrapper<T> {
        vtable: *mut IVRDisplayComponent__bindgen_vtable,
        display: T,
    }

    let wrapper = Box::new(DisplayWrapper {
        vtable: vtable_ptr,
        display,
    });

    Box::into_raw(wrapper) as *mut IVRDisplayComponent
}

pub fn create_display_wrapper(display: Arc<DisplayComponent>) -> *mut std::ffi::c_void {
    create_display_vtable(DisplayComponentWrapper(display)) as *mut std::ffi::c_void
}
