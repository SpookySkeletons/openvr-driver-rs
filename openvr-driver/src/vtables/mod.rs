//! Internal vtable generation for OpenVR interfaces
//!
//! This module handles all the low-level vtable creation and memory layout
//! required for OpenVR's C++ virtual interfaces. This is an internal module
//! and should not be used directly by driver developers.

mod device;
mod display;
mod provider;

pub(crate) use device::create_device_vtable;
pub(crate) use display::create_display_vtable;
pub(crate) use provider::create_provider_vtable;

use std::sync::Arc;

/// Internal vtable wrapper structure
///
/// This structure maintains the exact memory layout required by OpenVR:
/// - First field: pointer to vtable
/// - Second field: the actual data (Arc to maintain lifetime)
#[repr(C)]
pub(crate) struct VtableWrapper<V, T: ?Sized> {
    vtable: *mut V,
    data: Arc<T>,
}

impl<V, T: ?Sized> VtableWrapper<V, T> {
    /// Create a new vtable wrapper
    ///
    /// # Safety
    /// The vtable pointer must remain valid for the lifetime of this wrapper
    pub(crate) unsafe fn new(vtable: *mut V, data: Arc<T>) -> *mut Self {
        let wrapper = Box::new(Self { vtable, data });
        Box::into_raw(wrapper)
    }

    /// Recover the data from a vtable wrapper pointer
    ///
    /// # Safety
    /// The pointer must be a valid VtableWrapper created by `new`
    pub(crate) unsafe fn get_data<'a>(wrapper: *mut Self) -> &'a Arc<T> {
        &(*wrapper).data
    }

    /// Recover mutable data from a vtable wrapper pointer
    ///
    /// # Safety
    /// The pointer must be a valid VtableWrapper created by `new`
    pub(crate) unsafe fn get_data_mut<'a>(wrapper: *mut Self) -> &'a mut Arc<T> {
        &mut (*wrapper).data
    }
}

/// Helper macro to create thunk functions for vtable methods
///
/// This macro generates the unsafe extern "C" functions that OpenVR will call,
/// which then forward to the safe Rust trait implementations.
macro_rules! create_thunk {
    ($name:ident, $self_type:ty, $trait_method:ident, ($($arg:ident: $arg_type:ty),*) -> $ret:ty) => {
        unsafe extern "C" fn $name(
            this: *mut $self_type,
            $($arg: $arg_type),*
        ) -> $ret {
            let wrapper = this as *mut VtableWrapper<_, _>;
            let data = VtableWrapper::get_data(wrapper);
            data.$trait_method($($arg),*)
        }
    };

    ($name:ident, $self_type:ty, $trait_method:ident, ($($arg:ident: $arg_type:ty),*)) => {
        unsafe extern "C" fn $name(
            this: *mut $self_type,
            $($arg: $arg_type),*
        ) {
            let wrapper = this as *mut VtableWrapper<_, _>;
            let data = VtableWrapper::get_data(wrapper);
            data.$trait_method($($arg),*)
        }
    };
}

pub(crate) use create_thunk;

/// Helper to extract offset for data recovery
///
/// After the vtable pointer, we can find our Arc data
pub(crate) const VTABLE_OFFSET: usize = std::mem::size_of::<*mut std::ffi::c_void>();
