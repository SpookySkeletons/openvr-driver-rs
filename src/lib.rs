use openvr_driver_sys as sys;

// Re-export common types
pub use sys::vr::{ETrackedDeviceClass, EVREye, EVRInitError, TrackedDeviceIndex_t};

// Core traits
pub trait ServerTrackedDeviceProvider {
    fn init(&mut self, driver_context: &dyn DriverContext) -> Result<(), EVRInitError>;
    fn cleanup(&mut self);
    fn run_frame(&mut self);
    fn should_block_standby_mode(&self) -> bool;
    fn enter_standby(&mut self);
    fn leave_standby(&mut self);
}

pub trait TrackedDeviceServerDriver {
    fn activate(&mut self, device_id: TrackedDeviceIndex_t) -> Result<(), EVRInitError>;
    fn deactivate(&mut self);
    fn run_frame(&mut self);
    fn get_serial_number(&self) -> String;
    fn enter_standby(&mut self);
    fn get_device_class(&self) -> ETrackedDeviceClass;
}

pub trait DriverContext {
    fn get_generic_interface(&self, interface_version: &str) -> Option<*mut std::ffi::c_void>;
    fn get_driver_handle(&self) -> sys::vr::DriverHandle_t;
}
