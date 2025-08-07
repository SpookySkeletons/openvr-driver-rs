use crate::{
    display::{create_display_wrapper, DisplayComponent, DisplayComponentWrapper},
    settings::DriverSettings,
};
use openvr_driver_bindings::{
    interfaces::ITrackedDeviceServerDriver_Interface,
    root::vr::{DriverPose_t, EVRInitError, HmdQuaternion_t, VRInputComponentHandle_t},
};
use std::ffi::{c_char, c_void, CString};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// Newtype wrapper to avoid orphan rule
pub struct HmdDeviceWrapper(pub Arc<HmdDevice>);

// HMD Device Driver
pub struct HmdDevice {
    display_component: Arc<DisplayComponent>,
    settings: DriverSettings,
    input_handles: Vec<VRInputComponentHandle_t>,
    frame_number: AtomicU32,
    is_active: AtomicBool,
    device_index: AtomicU32,
    pose_thread: Mutex<Option<thread::JoinHandle<()>>>,
    should_stop: Arc<AtomicBool>,
}

unsafe impl Send for HmdDevice {}
unsafe impl Sync for HmdDevice {}

impl HmdDevice {
    pub fn new() -> Arc<Self> {
        // Load settings from VRSettings (or defaults)
        let settings = DriverSettings::load();

        // Create display configuration from settings
        let config = crate::DisplayConfiguration {
            window_x: settings.window_x,
            window_y: settings.window_y,
            window_width: settings.window_width,
            window_height: settings.window_height,
            render_width: settings.render_width,
            render_height: settings.render_height,
        };

        let display_component = DisplayComponent::new(config);

        Arc::new(Self {
            display_component,
            settings,
            input_handles: Vec::new(),
            frame_number: AtomicU32::new(0),
            is_active: AtomicBool::new(false),
            device_index: AtomicU32::new(0),
            pose_thread: Mutex::new(None),
            should_stop: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn get_serial_number(&self) -> &str {
        &self.settings.serial_number
    }

    pub fn run_frame(&self) {
        // Called every frame by the provider
        // In a real driver, you'd update device state here
    }

    pub fn deactivate(&self) {
        eprintln!("SimpleHMD: Deactivating device");

        self.is_active.store(false, Ordering::Relaxed);
        self.should_stop.store(true, Ordering::Relaxed);

        if let Some(thread) = self.pose_thread.lock().unwrap().take() {
            let _ = thread.join();
        }
    }
}

impl ITrackedDeviceServerDriver_Interface for HmdDeviceWrapper {
    fn activate(&self, device_index: u32) -> EVRInitError {
        eprintln!("SimpleHMD: Activating device {}", device_index);

        self.0.device_index.store(device_index, Ordering::Relaxed);
        self.0.is_active.store(true, Ordering::Relaxed);

        // Set device properties that OpenVR requires
        unsafe {
            // Get property container for this device
            // Note: In a real implementation, we'd get VRProperties from the driver context
            // For now, we'll just log that these properties should be set
            eprintln!("SimpleHMD: Should set device properties:");
            eprintln!("  - Model Number: {}", self.0.settings.model_number);
            eprintln!("  - Serial Number: {}", self.0.settings.serial_number);
            eprintln!(
                "  - Display Frequency: {} Hz",
                self.0.settings.display_frequency
            );
            eprintln!("  - IPD: {} meters", self.0.settings.ipd);
            eprintln!("  - Seconds from Vsync to Photons: 0.11");

            // TODO: When VRProperties interface is available:
            // let container = VRProperties()->TrackedDeviceToPropertyContainer(device_index);
            // VRProperties()->SetStringProperty(container, Prop_ModelNumber_String, model_number);
            // VRProperties()->SetStringProperty(container, Prop_SerialNumber_String, serial_number);
            // VRProperties()->SetFloatProperty(container, Prop_DisplayFrequency_Float, 90.0);
            // VRProperties()->SetFloatProperty(container, Prop_UserIpdMeters_Float, 0.063);
            // VRProperties()->SetFloatProperty(container, Prop_SecondsFromVsyncToPhotons_Float, 0.11);
            // VRProperties()->SetBoolProperty(container, Prop_IsOnDesktop_Bool, false);
            // VRProperties()->SetBoolProperty(container, Prop_DisplayDebugMode_Bool, true);
        }

        // Start pose update thread
        let should_stop = self.0.should_stop.clone();

        let pose_thread = thread::spawn(move || {
            while !should_stop.load(Ordering::Relaxed) {
                // Update pose at ~60Hz
                thread::sleep(Duration::from_millis(16));
                // In a real driver, you'd update the pose here
            }
        });

        *self.0.pose_thread.lock().unwrap() = Some(pose_thread);

        EVRInitError::None
    }

    fn deactivate(&self) {
        self.0.deactivate();
    }

    fn enter_standby(&self) {
        eprintln!("SimpleHMD: Device entering standby");
    }

    fn get_component(&self, component_name: *const c_char) -> *mut c_void {
        if component_name.is_null() {
            return std::ptr::null_mut();
        }

        unsafe {
            let name = std::ffi::CStr::from_ptr(component_name);
            let name_str = name.to_string_lossy();

            eprintln!("SimpleHMD: GetComponent requested for: {}", name_str);

            // Check if requesting display component
            // The IVRDisplayComponent is critical for HMDs to work properly
            if name_str == "IVRDisplayComponent_003" || name_str.starts_with("IVRDisplayComponent")
            {
                eprintln!("SimpleHMD: Returning display component");
                // Return the display component with its vtable
                return create_display_wrapper(self.0.display_component.clone());
            }
        }

        std::ptr::null_mut()
    }

    fn debug_request(
        &self,
        request: *const c_char,
        response_buffer: *mut c_char,
        response_buffer_size: u32,
    ) {
        if request.is_null() || response_buffer.is_null() || response_buffer_size == 0 {
            return;
        }

        unsafe {
            let request_str = std::ffi::CStr::from_ptr(request).to_string_lossy();
            eprintln!("SimpleHMD: Debug request: {}", request_str);

            let response = CString::new("OK").unwrap_or_else(|_| CString::new("").unwrap());
            let response_bytes = response.as_bytes_with_nul();
            let copy_len = response_bytes.len().min(response_buffer_size as usize);

            std::ptr::copy_nonoverlapping(
                response_bytes.as_ptr() as *const c_char,
                response_buffer,
                copy_len,
            );
        }
    }

    fn get_pose(&self) -> DriverPose_t {
        let mut pose = DriverPose_t {
            poseTimeOffset: 0.0,
            qWorldFromDriverRotation: HmdQuaternion_t {
                w: 1.0,
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            vecWorldFromDriverTranslation: [0.0, 0.0, 0.0],
            qDriverFromHeadRotation: HmdQuaternion_t {
                w: 1.0,
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            vecDriverFromHeadTranslation: [0.0, 0.0, 0.0],
            vecPosition: [0.0, 1.6, 0.0], // 1.6m height
            vecVelocity: [0.0, 0.0, 0.0],
            vecAcceleration: [0.0, 0.0, 0.0],
            qRotation: HmdQuaternion_t {
                w: 1.0,
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            vecAngularVelocity: [0.0, 0.0, 0.0],
            vecAngularAcceleration: [0.0, 0.0, 0.0],
            result: unsafe { std::mem::transmute(200i32) }, // TrackingResult_Running_OK
            poseIsValid: true,
            willDriftInYaw: false,
            shouldApplyHeadModel: false,
            deviceIsConnected: true,
        };

        // Simple animation: rotate the HMD slowly
        let frame = self.0.frame_number.fetch_add(1, Ordering::Relaxed) as f64;
        let angle = frame * 0.01;

        // Quaternion for Y-axis rotation
        pose.qRotation.w = (angle / 2.0).cos();
        pose.qRotation.x = 0.0;
        pose.qRotation.y = (angle / 2.0).sin();
        pose.qRotation.z = 0.0;

        pose
    }
}

// Helper to create device vtable
pub fn create_device_vtable<T: ITrackedDeviceServerDriver_Interface + 'static>(
    device: T,
) -> *mut openvr_driver_bindings::root::vr::ITrackedDeviceServerDriver {
    use openvr_driver_bindings::root::vr::{
        ITrackedDeviceServerDriver, ITrackedDeviceServerDriver__bindgen_vtable,
    };

    // Create thunk functions for the vtable
    unsafe extern "C" fn activate_thunk<T: ITrackedDeviceServerDriver_Interface>(
        this: *mut ITrackedDeviceServerDriver,
        device_index: u32,
    ) -> EVRInitError {
        // Recover the device from the pointer stored after the vtable
        let device_ptr = (this as *mut u8).add(std::mem::size_of::<
            *mut ITrackedDeviceServerDriver__bindgen_vtable,
        >()) as *mut T;
        let device = &*device_ptr;
        device.activate(device_index)
    }

    unsafe extern "C" fn deactivate_thunk<T: ITrackedDeviceServerDriver_Interface>(
        this: *mut ITrackedDeviceServerDriver,
    ) {
        let device_ptr = (this as *mut u8).add(std::mem::size_of::<
            *mut ITrackedDeviceServerDriver__bindgen_vtable,
        >()) as *mut T;
        let device = &*device_ptr;
        device.deactivate();
    }

    unsafe extern "C" fn enter_standby_thunk<T: ITrackedDeviceServerDriver_Interface>(
        this: *mut ITrackedDeviceServerDriver,
    ) {
        let device_ptr = (this as *mut u8).add(std::mem::size_of::<
            *mut ITrackedDeviceServerDriver__bindgen_vtable,
        >()) as *mut T;
        let device = &*device_ptr;
        device.enter_standby();
    }

    unsafe extern "C" fn get_component_thunk<T: ITrackedDeviceServerDriver_Interface>(
        this: *mut ITrackedDeviceServerDriver,
        component_name: *const c_char,
    ) -> *mut c_void {
        let device_ptr = (this as *mut u8).add(std::mem::size_of::<
            *mut ITrackedDeviceServerDriver__bindgen_vtable,
        >()) as *mut T;
        let device = &*device_ptr;
        device.get_component(component_name)
    }

    unsafe extern "C" fn debug_request_thunk<T: ITrackedDeviceServerDriver_Interface>(
        this: *mut ITrackedDeviceServerDriver,
        request: *const c_char,
        response_buffer: *mut c_char,
        response_buffer_size: u32,
    ) {
        let device_ptr = (this as *mut u8).add(std::mem::size_of::<
            *mut ITrackedDeviceServerDriver__bindgen_vtable,
        >()) as *mut T;
        let device = &*device_ptr;
        device.debug_request(request, response_buffer, response_buffer_size);
    }

    unsafe extern "C" fn get_pose_thunk<T: ITrackedDeviceServerDriver_Interface>(
        this: *mut ITrackedDeviceServerDriver,
    ) -> DriverPose_t {
        let device_ptr = (this as *mut u8).add(std::mem::size_of::<
            *mut ITrackedDeviceServerDriver__bindgen_vtable,
        >()) as *mut T;
        let device = &*device_ptr;
        device.get_pose()
    }

    // Create the vtable
    let vtable = Box::new(ITrackedDeviceServerDriver__bindgen_vtable {
        ITrackedDeviceServerDriver_Activate: activate_thunk::<T>,
        ITrackedDeviceServerDriver_Deactivate: deactivate_thunk::<T>,
        ITrackedDeviceServerDriver_EnterStandby: enter_standby_thunk::<T>,
        ITrackedDeviceServerDriver_GetComponent: get_component_thunk::<T>,
        ITrackedDeviceServerDriver_DebugRequest: debug_request_thunk::<T>,
        ITrackedDeviceServerDriver_GetPose: get_pose_thunk::<T>,
    });
    let vtable_ptr = Box::into_raw(vtable);

    // Create a custom structure that stores both vtable pointer and device
    #[repr(C)]
    struct DeviceWrapper<T> {
        vtable: *mut ITrackedDeviceServerDriver__bindgen_vtable,
        device: T,
    }

    let wrapper = Box::new(DeviceWrapper {
        vtable: vtable_ptr,
        device,
    });

    Box::into_raw(wrapper) as *mut ITrackedDeviceServerDriver
}

pub fn create_device_wrapper(device: Arc<HmdDevice>) -> *mut c_void {
    create_device_vtable(HmdDeviceWrapper(device)) as *mut c_void
}
