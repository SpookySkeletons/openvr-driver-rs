use crate::DisplayConfiguration;
use openvr_driver_bindings::{
    interfaces::ITrackedDeviceServerDriver_Interface,
    root::vr::{DriverPose_t, EVRInitError, HmdQuaternion_t, VRInputComponentHandle_t},
};
use std::ffi::{c_char, c_void, CString};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// Display component for the HMD
pub struct DisplayComponent {
    config: DisplayConfiguration,
}

impl DisplayComponent {
    pub fn new(config: DisplayConfiguration) -> Self {
        Self { config }
    }
}

// HMD Device Driver
pub struct HmdDevice {
    display_component: Arc<Mutex<DisplayComponent>>,
    model_number: String,
    serial_number: String,
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
        let config = DisplayConfiguration::default();

        Arc::new(Self {
            display_component: Arc::new(Mutex::new(DisplayComponent::new(config))),
            model_number: "SimpleHMD_Model_v1".to_string(),
            serial_number: "SIMPLEHMD_001".to_string(),
            input_handles: Vec::new(),
            frame_number: AtomicU32::new(0),
            is_active: AtomicBool::new(false),
            device_index: AtomicU32::new(0),
            pose_thread: Mutex::new(None),
            should_stop: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn get_serial_number(&self) -> &str {
        &self.serial_number
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

impl ITrackedDeviceServerDriver_Interface for Arc<HmdDevice> {
    fn activate(&self, device_index: u32) -> EVRInitError {
        eprintln!("SimpleHMD: Activating device {}", device_index);

        self.device_index.store(device_index, Ordering::Relaxed);
        self.is_active.store(true, Ordering::Relaxed);

        // Start pose update thread
        let should_stop = self.should_stop.clone();

        let pose_thread = thread::spawn(move || {
            while !should_stop.load(Ordering::Relaxed) {
                // Update pose at ~60Hz
                thread::sleep(Duration::from_millis(16));
                // In a real driver, you'd update the pose here
            }
        });

        *self.pose_thread.lock().unwrap() = Some(pose_thread);

        EVRInitError::None
    }

    fn deactivate(&self) {
        self.deactivate();
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
            if name_str.starts_with("IVRDisplayComponent") {
                // TODO: Return display component vtable
                eprintln!("SimpleHMD: Would return display component here");
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
        let frame = self.frame_number.fetch_add(1, Ordering::Relaxed) as f64;
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
    device: Arc<T>,
) -> *mut openvr_driver_bindings::root::vr::ITrackedDeviceServerDriver {
    use openvr_driver_bindings::root::vr::{
        ITrackedDeviceServerDriver, ITrackedDeviceServerDriver__bindgen_vtable,
    };

    // Store the device Arc in a leaked box to keep it alive
    let device_ptr = Box::into_raw(Box::new(device));

    // Create thunk functions for the vtable
    unsafe extern "C" fn activate_thunk<T: ITrackedDeviceServerDriver_Interface>(
        this: *mut ITrackedDeviceServerDriver,
        device_index: u32,
    ) -> EVRInitError {
        // Recover the device from the pointer stored after the vtable
        let device_ptr = (this as *mut u8).add(std::mem::size_of::<
            *mut ITrackedDeviceServerDriver__bindgen_vtable,
        >()) as *mut *mut Arc<T>;
        let device = &**device_ptr;
        device.activate(device_index)
    }

    unsafe extern "C" fn deactivate_thunk<T: ITrackedDeviceServerDriver_Interface>(
        this: *mut ITrackedDeviceServerDriver,
    ) {
        let device_ptr = (this as *mut u8).add(std::mem::size_of::<
            *mut ITrackedDeviceServerDriver__bindgen_vtable,
        >()) as *mut *mut Arc<T>;
        let device = &**device_ptr;
        device.deactivate();
    }

    unsafe extern "C" fn enter_standby_thunk<T: ITrackedDeviceServerDriver_Interface>(
        this: *mut ITrackedDeviceServerDriver,
    ) {
        let device_ptr = (this as *mut u8).add(std::mem::size_of::<
            *mut ITrackedDeviceServerDriver__bindgen_vtable,
        >()) as *mut *mut Arc<T>;
        let device = &**device_ptr;
        device.enter_standby();
    }

    unsafe extern "C" fn get_component_thunk<T: ITrackedDeviceServerDriver_Interface>(
        this: *mut ITrackedDeviceServerDriver,
        component_name: *const c_char,
    ) -> *mut c_void {
        let device_ptr = (this as *mut u8).add(std::mem::size_of::<
            *mut ITrackedDeviceServerDriver__bindgen_vtable,
        >()) as *mut *mut Arc<T>;
        let device = &**device_ptr;
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
        >()) as *mut *mut Arc<T>;
        let device = &**device_ptr;
        device.debug_request(request, response_buffer, response_buffer_size);
    }

    unsafe extern "C" fn get_pose_thunk<T: ITrackedDeviceServerDriver_Interface>(
        this: *mut ITrackedDeviceServerDriver,
    ) -> DriverPose_t {
        let device_ptr = (this as *mut u8).add(std::mem::size_of::<
            *mut ITrackedDeviceServerDriver__bindgen_vtable,
        >()) as *mut *mut Arc<T>;
        let device = &**device_ptr;
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
        device: *mut Arc<T>,
    }

    let wrapper = Box::new(DeviceWrapper {
        vtable: vtable_ptr,
        device: device_ptr,
    });

    Box::into_raw(wrapper) as *mut ITrackedDeviceServerDriver
}

pub fn create_device_wrapper(device: Arc<HmdDevice>) -> *mut c_void {
    create_device_vtable(device) as *mut c_void
}
