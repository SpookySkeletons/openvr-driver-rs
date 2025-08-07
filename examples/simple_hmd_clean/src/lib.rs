//! Simple HMD Driver Example - Clean Implementation
//!
//! This example demonstrates how to create an OpenVR HMD driver using the
//! high-level openvr-driver library, without any vtable boilerplate.

use openvr_driver::prelude::*;
use openvr_driver::{
    settings, ComponentResult, DisplayConfiguration, DriverContext, DriverPose, DriverResult, Eye,
    HmdMatrix34, HmdQuaternion, InitError, PropertyContainer, ServerTrackedDeviceProvider,
    TrackedDeviceServerDriver,
};
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

/// Main driver provider
pub struct SimpleHmdProvider {
    devices: Vec<Arc<HmdDeviceWrapper>>,
}

impl Default for SimpleHmdProvider {
    fn default() -> Self {
        Self {
            devices: Vec::new(),
        }
    }
}

impl ServerTrackedDeviceProvider for SimpleHmdProvider {
    fn init(&mut self, context: &mut DriverContext) -> DriverResult<()> {
        eprintln!("SimpleHMD: Initializing provider");

        // Create and register our HMD device
        let device = Arc::new(HmdDeviceWrapper::new());
        eprintln!(
            "SimpleHMD: Created HMD device with serial: {}",
            device.get_serial_number()
        );

        // Register the device with OpenVR
        eprintln!("SimpleHMD: Registering device with OpenVR...");
        match context.register_device(device.clone() as Arc<dyn TrackedDeviceServerDriver>) {
            Ok(()) => {
                eprintln!("SimpleHMD: Device registered successfully");
                eprintln!("SimpleHMD: Waiting for OpenVR to activate device...");

                // Store reference to the device so we can manage it
                self.devices.push(device);
            }
            Err(e) => {
                eprintln!("SimpleHMD: Failed to register device: {:?}", e);
                return Err(e);
            }
        }

        eprintln!(
            "SimpleHMD: Provider initialized with {} device(s)",
            self.devices.len()
        );
        Ok(())
    }

    fn cleanup(&mut self) {
        eprintln!("SimpleHMD: Cleaning up provider");

        // Stop all devices
        for device in &self.devices {
            device.stop();
        }

        self.devices.clear();
    }

    fn run_frame(&mut self) {
        // Check for pending activation from vtable callback
        if let Some(device_index) = openvr_driver::take_pending_device_activation() {
            eprintln!(
                "SimpleHMD: Processing pending activation for device index {} in run_frame",
                device_index
            );

            // Activate the first device (we only have one HMD)
            if let Some(device) = self.devices.first() {
                if let Err(e) = device.perform_activation(device_index) {
                    eprintln!(
                        "SimpleHMD: ERROR: Failed to activate device {}: {:?}",
                        device_index, e
                    );
                }
            }
        }

        // Update all devices each frame
        for device in &self.devices {
            device.update_pose();
        }
    }

    fn should_block_standby_mode(&self) -> bool {
        // Don't block standby for this simple example
        false
    }

    fn enter_standby(&mut self) {
        eprintln!("SimpleHMD: Entering standby");
    }

    fn leave_standby(&mut self) {
        eprintln!("SimpleHMD: Leaving standby");
    }
}

/// HMD Device wrapper for TrackedDeviceServerDriver implementation
pub struct HmdDeviceWrapper(Arc<Mutex<HmdDeviceInner>>);

/// HMD Device implementation
pub struct HmdDeviceInner {
    config: DeviceConfig,
    display: Arc<DisplayComponentImpl>,
    device_index: AtomicU32,
    is_active: AtomicBool,
    frame_counter: AtomicU32,
}

/// Device configuration loaded from settings
#[derive(Clone)]
struct DeviceConfig {
    serial_number: String,
    model_number: String,
    window_x: i32,
    window_y: i32,
    window_width: i32,
    window_height: i32,
    render_width: i32,
    render_height: i32,
    display_frequency: f32,
    ipd: f32,
    seconds_from_vsync_to_photons: f32,
}

impl Default for DeviceConfig {
    fn default() -> Self {
        Self {
            serial_number: settings::get_string(
                "driver_simple_hmd_clean",
                "serial_number",
                "SIMPLEHMD_001",
            ),
            model_number: settings::get_string(
                "driver_simple_hmd_clean",
                "model_number",
                "SimpleHMD Model 1",
            ),
            window_x: settings::get_int32("simple_hmd_clean_display", "window_x", 0),
            window_y: settings::get_int32("simple_hmd_clean_display", "window_y", 0),
            window_width: settings::get_int32("simple_hmd_clean_display", "window_width", 1920),
            window_height: settings::get_int32("simple_hmd_clean_display", "window_height", 1080),
            render_width: settings::get_int32("simple_hmd_clean_display", "render_width", 1920),
            render_height: settings::get_int32("simple_hmd_clean_display", "render_height", 1080),
            display_frequency: settings::get_float(
                "simple_hmd_clean_display",
                "display_frequency",
                90.0,
            ),
            ipd: settings::get_float("simple_hmd_clean_display", "ipd", 0.063),
            seconds_from_vsync_to_photons: settings::get_float(
                "simple_hmd_clean_display",
                "seconds_from_vsync_to_photons",
                0.011,
            ),
        }
    }
}

impl HmdDeviceWrapper {
    fn new() -> Self {
        let config = DeviceConfig::default();

        let display_config = DisplayConfiguration {
            window_x: config.window_x,
            window_y: config.window_y,
            window_width: config.window_width,
            window_height: config.window_height,
            render_width: config.render_width,
            render_height: config.render_height,
            display_frequency: config.display_frequency,
            ipd: config.ipd,
            seconds_from_vsync_to_photons: config.seconds_from_vsync_to_photons,
        };

        Self(Arc::new(Mutex::new(HmdDeviceInner {
            display: Arc::new(DisplayComponentImpl::new(display_config)),
            config,
            device_index: AtomicU32::new(0),
            is_active: AtomicBool::new(false),
            frame_counter: AtomicU32::new(0),
        })))
    }

    fn stop(&self) {
        let inner = self.0.lock();
        inner.is_active.store(false, Ordering::Relaxed);
    }

    fn update_pose(&self) {
        let inner = self.0.lock();
        // Update animation frame counter
        inner.frame_counter.fetch_add(1, Ordering::Relaxed);
    }
}

impl TrackedDeviceServerDriver for HmdDeviceWrapper {
    fn perform_activation(&self, device_index: u32) -> DriverResult<()> {
        eprintln!(
            "SimpleHMD: [ACTIVATION] Performing activation for device index {}",
            device_index
        );

        // Store the device index
        let inner = self.0.lock();
        inner.device_index.store(device_index, Ordering::Relaxed);
        inner.is_active.store(true, Ordering::Relaxed);

        // Get values needed for properties before dropping the lock
        let model = inner.config.model_number.clone();
        let serial = inner.config.serial_number.clone();
        let freq = inner.config.display_frequency;
        let ipd = inner.config.ipd;
        eprintln!(
            "SimpleHMD: [ACTIVATION] About to set properties - Model: {}, Serial: {}, Freq: {}Hz, IPD: {}m",
            model, serial, freq, ipd
        );
        drop(inner); // Release lock before calling external functions

        // Set device properties using batch writing
        eprintln!("SimpleHMD: [ACTIVATION] Calling set_hmd_properties...");
        let result =
            openvr_driver::properties::set_hmd_properties(device_index, &model, &serial, freq, ipd);

        match &result {
            Ok(()) => {
                eprintln!(
                    "SimpleHMD: [ACTIVATION] Device {} activated and properties set successfully",
                    device_index
                );
            }
            Err(e) => {
                eprintln!(
                    "SimpleHMD: [ACTIVATION] ERROR: Failed to set properties for device {}: {:?}",
                    device_index, e
                );
            }
        }

        result
    }

    fn get_serial_number(&self) -> String {
        let inner = self.0.lock();
        inner.config.serial_number.clone()
    }

    fn activate(&mut self, device_index: u32) -> DriverResult<()> {
        eprintln!(
            "SimpleHMD: Device activate callback received for index {} (handled by vtable)",
            device_index
        );
        // The vtable handles storing the activation index globally
        // It will be processed in run_frame
        Ok(())
    }

    fn deactivate(&mut self) {
        eprintln!("SimpleHMD: Deactivating device");
        let inner = self.0.lock();
        inner.is_active.store(false, Ordering::Relaxed);
    }

    fn enter_standby(&mut self) {
        eprintln!("SimpleHMD: Device entering standby");
    }

    fn get_component(&self, component_name: &str) -> ComponentResult {
        eprintln!("SimpleHMD: GetComponent requested for: {}", component_name);

        match component_name {
            "IVRDisplayComponent_003" => {
                eprintln!(
                    "SimpleHMD: Creating and returning display component vtable for version 003"
                );
                let inner = self.0.lock();
                // Create the display component vtable
                let display_vtable = openvr_driver::Component::create_vtable(inner.display.clone());
                eprintln!(
                    "SimpleHMD: Display component vtable created at {:?}",
                    display_vtable
                );
                Some(display_vtable)
            }
            _ => {
                eprintln!("SimpleHMD: Component {} not implemented", component_name);
                None
            }
        }
    }

    fn debug_request(&mut self, request: &str) -> String {
        format!("SimpleHMD Debug: Received request '{}'", request)
    }

    fn get_pose(&self) -> DriverPose {
        // Create a basic pose with some animation
        let frame = {
            let inner = self.0.lock();
            inner.frame_counter.load(Ordering::Relaxed) as f64
        };
        let angle = frame * 0.01; // Slow rotation

        // Create quaternion for Y-axis rotation
        let quat = HmdQuaternion {
            w: (angle / 2.0).cos(),
            x: 0.0,
            y: (angle / 2.0).sin(),
            z: 0.0,
        };

        DriverPose {
            poseTimeOffset: 0.0,
            qWorldFromDriverRotation: HmdQuaternion {
                w: 1.0,
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            vecWorldFromDriverTranslation: [0.0, 0.0, 0.0],
            qDriverFromHeadRotation: HmdQuaternion {
                w: 1.0,
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            vecDriverFromHeadTranslation: [0.0, 0.0, 0.0],
            vecPosition: [0.0, 1.6, -1.0], // Head height position
            vecVelocity: [0.0, 0.0, 0.0],
            vecAcceleration: [0.0, 0.0, 0.0],
            qRotation: quat,
            vecAngularVelocity: [0.0, 0.01, 0.0],
            vecAngularAcceleration: [0.0, 0.0, 0.0],
            result: unsafe { std::mem::transmute(200i32) }, // TrackingResult_Running_OK
            poseIsValid: true,
            willDriftInYaw: false,
            shouldApplyHeadModel: false,
            deviceIsConnected: true,
        }
    }
}

/// Display component implementation
struct DisplayComponentImpl {
    config: DisplayConfiguration,
}

impl DisplayComponentImpl {
    fn new(config: DisplayConfiguration) -> Self {
        Self { config }
    }
}

impl openvr_driver::DisplayComponent for DisplayComponentImpl {
    fn get_window_bounds(&self) -> (i32, i32, i32, i32) {
        (
            self.config.window_x,
            self.config.window_y,
            self.config.window_width,
            self.config.window_height,
        )
    }

    fn get_recommended_render_target_size(&self) -> (u32, u32) {
        (
            self.config.render_width as u32,
            self.config.render_height as u32,
        )
    }

    fn get_eye_to_head_transform(&self, eye: Eye) -> HmdMatrix34 {
        let ipd_half = self.config.ipd / 2.0;
        let x_offset = match eye {
            Eye::Left => -ipd_half,
            Eye::Right => ipd_half,
        };

        HmdMatrix34 {
            m: [
                [1.0, 0.0, 0.0, x_offset],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
            ],
        }
    }

    fn is_display_on_desktop(&self) -> bool {
        true // Match C++ implementation
    }

    fn is_display_real(&self) -> bool {
        false // This is a virtual HMD, not a real display
    }
}

// Entry point - this is all that's needed!
openvr_driver_entry!(SimpleHmdProvider);
