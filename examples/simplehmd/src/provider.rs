use crate::{DisplayConfiguration, HmdDevice};
use openvr_driver_bindings::{
    interfaces::IServerTrackedDeviceProvider_Interface,
    root::vr::{EVRInitError, IVRDriverContext, IVRServerDriverHost},
};
use std::ffi::{c_char, c_void, CStr, CString};
use std::sync::{Arc, Mutex};

pub struct SimpleHmdProvider {
    hmd_device: Option<Arc<HmdDevice>>,
    driver_context: Option<*mut IVRDriverContext>,
    driver_host: Option<*mut IVRServerDriverHost>,
    interface_versions: Vec<CString>,
    interface_versions_ptrs: Vec<*const c_char>,
}

unsafe impl Send for SimpleHmdProvider {}
unsafe impl Sync for SimpleHmdProvider {}

impl SimpleHmdProvider {
    pub fn new() -> Arc<Mutex<Self>> {
        let interface_versions = vec![
            CString::new("IServerTrackedDeviceProvider_005").unwrap(),
            CString::new("ITrackedDeviceServerDriver_005").unwrap(),
        ];

        let mut interface_versions_ptrs: Vec<*const c_char> =
            interface_versions.iter().map(|s| s.as_ptr()).collect();
        interface_versions_ptrs.push(std::ptr::null());

        Arc::new(Mutex::new(Self {
            hmd_device: None,
            driver_context: None,
            driver_host: None,
            interface_versions,
            interface_versions_ptrs,
        }))
    }

    pub fn get_driver_context(&self) -> Option<*mut IVRDriverContext> {
        self.driver_context
    }

    pub fn get_driver_host(&self) -> Option<*mut IVRServerDriverHost> {
        self.driver_host
    }
}

impl IServerTrackedDeviceProvider_Interface for Arc<Mutex<SimpleHmdProvider>> {
    fn init(&self, driver_context: *mut c_void) -> EVRInitError {
        eprintln!("SimpleHMD Provider: Initializing...");

        let mut provider = self.lock().unwrap();
        provider.driver_context = Some(driver_context as *mut IVRDriverContext);

        // Get the driver host interface for registering devices
        if !driver_context.is_null() {
            unsafe {
                let context = driver_context as *mut IVRDriverContext;
                let host_interface = CString::new("IVRServerDriverHost_006").unwrap();
                let mut error = EVRInitError::None;

                let host_ptr = (*context)
                    .GetGenericInterface(host_interface.as_ptr(), &mut error as *mut _ as *mut i32);

                if !host_ptr.is_null() && error == EVRInitError::None {
                    provider.driver_host = Some(host_ptr as *mut IVRServerDriverHost);
                    eprintln!("SimpleHMD Provider: Got driver host interface");

                    // Create and register the HMD device
                    let hmd = HmdDevice::new();

                    // Register the device with OpenVR
                    let serial = CString::new(hmd.get_serial_number()).unwrap();
                    let device_class = openvr_driver_bindings::root::vr::ETrackedDeviceClass::HMD;

                    // TODO: We need to create a device wrapper and register it
                    // For now, just store the device
                    provider.hmd_device = Some(hmd);

                    eprintln!("SimpleHMD Provider: Created HMD device");
                } else {
                    eprintln!("SimpleHMD Provider: Failed to get driver host interface");
                    return EVRInitError::Init_InterfaceNotFound;
                }
            }
        }

        eprintln!("SimpleHMD Provider: Initialization complete");
        EVRInitError::None
    }

    fn cleanup(&self) {
        eprintln!("SimpleHMD Provider: Cleaning up...");

        let mut provider = self.lock().unwrap();

        if let Some(hmd) = &provider.hmd_device {
            hmd.deactivate();
        }

        provider.hmd_device = None;
        provider.driver_context = None;
        provider.driver_host = None;
    }

    fn get_interface_versions(&self) -> *const *const c_char {
        let provider = self.lock().unwrap();
        provider.interface_versions_ptrs.as_ptr()
    }

    fn run_frame(&self) {
        let provider = self.lock().unwrap();
        if let Some(hmd) = &provider.hmd_device {
            hmd.run_frame();
        }
    }

    fn should_block_standby_mode(&self) -> bool {
        false
    }

    fn enter_standby(&self) {
        eprintln!("SimpleHMD Provider: Entering standby");
    }

    fn leave_standby(&self) {
        eprintln!("SimpleHMD Provider: Leaving standby");
    }
}
