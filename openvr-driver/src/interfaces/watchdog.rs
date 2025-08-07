//! Watchdog Provider interface
//!
//! This interface is implemented by drivers that need to monitor
//! system health and recover from failures.

use crate::DriverResult;

/// Watchdog provider interface for system health monitoring
///
/// Implement this trait to provide watchdog functionality that monitors
/// driver health and can trigger recovery actions when issues are detected.
pub trait WatchdogProvider: Send + Sync + 'static {
    /// Initialize the watchdog
    ///
    /// Called when the watchdog system is being set up.
    fn init(&mut self) -> DriverResult<()> {
        Ok(())
    }

    /// Cleanup the watchdog
    ///
    /// Called when the watchdog is being shut down.
    fn cleanup(&mut self) {
        // Default implementation does nothing
    }

    /// Called periodically to check system health
    ///
    /// This method should perform health checks and return whether
    /// the system is healthy. If false is returned too many times,
    /// OpenVR may trigger recovery actions.
    ///
    /// # Returns
    /// * `true` if the system is healthy
    /// * `false` if there are issues that need attention
    fn is_healthy(&self) -> bool {
        true
    }

    /// Wake up the watchdog thread
    ///
    /// Called when the watchdog needs to perform immediate checks.
    fn wake_up(&mut self) {
        // Default implementation does nothing
    }

    /// Get the watchdog timeout in seconds
    ///
    /// Returns how long the watchdog should wait before considering
    /// the driver unresponsive.
    ///
    /// # Returns
    /// * Timeout in seconds (default: 10.0)
    fn get_timeout_seconds(&self) -> f32 {
        10.0
    }

    /// Handle a watchdog timeout
    ///
    /// Called when the watchdog timeout has been exceeded.
    /// The driver can attempt recovery actions here.
    ///
    /// # Returns
    /// * `true` if recovery was successful
    /// * `false` if the driver should be restarted
    fn handle_timeout(&mut self) -> bool {
        false
    }
}
