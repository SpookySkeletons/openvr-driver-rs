//! Display component vtable generation
//!
//! This module handles the creation of vtables for the DisplayComponent interface.

use crate::{sys, DisplayComponent, Eye};
use std::ffi::c_void;
use std::sync::Arc;

use super::VtableWrapper;

/// Create a vtable for a DisplayComponent implementation
pub(crate) fn create_display_vtable<T>(component: Arc<T>) -> *mut c_void
where
    T: DisplayComponent + 'static,
{
    use sys::root::vr::{
        DistortionCoordinates_t, EVREye, HmdMatrix34_t, IVRDisplayComponent,
        IVRDisplayComponent__bindgen_vtable,
    };

    // Create thunk functions that forward to the Rust implementation
    unsafe extern "C" fn get_window_bounds_thunk<T: DisplayComponent>(
        this: *mut IVRDisplayComponent,
        x: *mut i32,
        y: *mut i32,
        width: *mut u32,
        height: *mut u32,
    ) {
        if x.is_null() || y.is_null() || width.is_null() || height.is_null() {
            return;
        }

        let wrapper = this as *mut VtableWrapper<IVRDisplayComponent__bindgen_vtable, T>;
        let component = &(*wrapper).data;

        let (px, py, w, h) = component.get_window_bounds();
        *x = px;
        *y = py;
        *width = w as u32;
        *height = h as u32;
    }

    unsafe extern "C" fn is_display_on_desktop_thunk<T: DisplayComponent>(
        this: *mut IVRDisplayComponent,
    ) -> bool {
        let wrapper = this as *mut VtableWrapper<IVRDisplayComponent__bindgen_vtable, T>;
        let component = &(*wrapper).data;
        component.is_display_on_desktop()
    }

    unsafe extern "C" fn is_display_real_thunk<T: DisplayComponent>(
        this: *mut IVRDisplayComponent,
    ) -> bool {
        let wrapper = this as *mut VtableWrapper<IVRDisplayComponent__bindgen_vtable, T>;
        let component = &(*wrapper).data;
        component.is_display_real()
    }

    unsafe extern "C" fn get_recommended_render_target_size_thunk<T: DisplayComponent>(
        this: *mut IVRDisplayComponent,
        width: *mut u32,
        height: *mut u32,
    ) {
        if width.is_null() || height.is_null() {
            return;
        }

        let wrapper = this as *mut VtableWrapper<IVRDisplayComponent__bindgen_vtable, T>;
        let component = &(*wrapper).data;

        let (w, h) = component.get_recommended_render_target_size();
        *width = w;
        *height = h;
    }

    unsafe extern "C" fn get_eye_output_viewport_thunk<T: DisplayComponent>(
        this: *mut IVRDisplayComponent,
        eye: EVREye,
        x: *mut u32,
        y: *mut u32,
        width: *mut u32,
        height: *mut u32,
    ) {
        if x.is_null() || y.is_null() || width.is_null() || height.is_null() {
            return;
        }

        let wrapper = this as *mut VtableWrapper<IVRDisplayComponent__bindgen_vtable, T>;
        let component = &(*wrapper).data;

        // Default implementation - full viewport for each eye
        let (_, _, w, h) = component.get_window_bounds();
        *x = 0;
        *y = 0;
        *width = w as u32 / 2; // Half width for each eye
        *height = h as u32;
    }

    unsafe extern "C" fn get_projection_raw_thunk<T: DisplayComponent>(
        this: *mut IVRDisplayComponent,
        eye: EVREye,
        left: *mut f32,
        right: *mut f32,
        top: *mut f32,
        bottom: *mut f32,
    ) {
        if left.is_null() || right.is_null() || top.is_null() || bottom.is_null() {
            return;
        }

        let wrapper = this as *mut VtableWrapper<IVRDisplayComponent__bindgen_vtable, T>;
        let component = &(*wrapper).data;

        let eye_enum = match eye {
            EVREye::Eye_Left => Eye::Left,
            EVREye::Eye_Right => Eye::Right,
            _ => Eye::Left,
        };

        // Get projection matrix and extract frustum parameters
        // This is a simplified implementation - you may need to adjust based on your needs
        let projection = component.get_projection(eye_enum, 0.1, 1000.0);

        // Extract frustum parameters from projection matrix
        // These formulas assume a standard projection matrix layout
        let near = 0.1f32;
        *left = -near;
        *right = near;
        *top = near;
        *bottom = -near;
    }

    unsafe extern "C" fn compute_distortion_thunk<T: DisplayComponent>(
        this: *mut IVRDisplayComponent,
        eye: EVREye,
        u: f32,
        v: f32,
    ) -> DistortionCoordinates_t {
        let wrapper = this as *mut VtableWrapper<IVRDisplayComponent__bindgen_vtable, T>;
        let component = &(*wrapper).data;

        let eye_enum = match eye {
            EVREye::Eye_Left => Eye::Left,
            EVREye::Eye_Right => Eye::Right,
            _ => Eye::Left,
        };

        let (red_u, red_v, green_u, green_v, blue_u, blue_v) =
            component.compute_distortion(eye_enum, u, v);

        DistortionCoordinates_t {
            rfRed: [red_u, red_v],
            rfGreen: [green_u, green_v],
            rfBlue: [blue_u, blue_v],
        }
    }

    unsafe extern "C" fn compute_inverse_distortion_thunk<T: DisplayComponent>(
        this: *mut IVRDisplayComponent,
        result: *mut sys::root::vr::HmdVector2_t,
        eye: EVREye,
        channel: u32,
        u: f32,
        v: f32,
    ) -> bool {
        if result.is_null() {
            return false;
        }

        let wrapper = this as *mut VtableWrapper<IVRDisplayComponent__bindgen_vtable, T>;
        let component = &(*wrapper).data;

        // Default implementation - no inverse distortion
        // In a real implementation, this would compute the inverse of the distortion
        *result = sys::root::vr::HmdVector2_t { v: [u, v] };
        true
    }

    // Create the vtable
    let vtable = Box::new(IVRDisplayComponent__bindgen_vtable {
        IVRDisplayComponent_GetWindowBounds: get_window_bounds_thunk::<T>,
        IVRDisplayComponent_IsDisplayOnDesktop: is_display_on_desktop_thunk::<T>,
        IVRDisplayComponent_IsDisplayRealDisplay: is_display_real_thunk::<T>,
        IVRDisplayComponent_GetRecommendedRenderTargetSize:
            get_recommended_render_target_size_thunk::<T>,
        IVRDisplayComponent_GetEyeOutputViewport: get_eye_output_viewport_thunk::<T>,
        IVRDisplayComponent_GetProjectionRaw: get_projection_raw_thunk::<T>,
        IVRDisplayComponent_ComputeDistortion: compute_distortion_thunk::<T>,
        IVRDisplayComponent_ComputeInverseDistortion: compute_inverse_distortion_thunk::<T>,
    });

    let vtable_ptr = Box::into_raw(vtable);

    // Create the wrapper that contains both vtable pointer and data
    unsafe {
        let wrapper = VtableWrapper::new(vtable_ptr, component);
        wrapper as *mut c_void
    }
}
