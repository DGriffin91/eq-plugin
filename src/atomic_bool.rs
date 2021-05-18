use std::sync::atomic::Ordering;

/// Simple atomic floating point variable with relaxed ordering.
///
/// Designed for the common case of sharing VST parameters between
/// multiple threads when no synchronization or change notification
/// is needed.
pub struct AtomicBool {
    atomic: std::sync::atomic::AtomicBool,
}

impl AtomicBool {
    /// New atomic float with initial value `value`.
    pub fn new(value: bool) -> AtomicBool {
        AtomicBool {
            atomic: std::sync::atomic::AtomicBool::new(value),
        }
    }

    /// Get the current value of the atomic float.
    pub fn get(&self) -> bool {
        self.atomic.load(Ordering::Relaxed)
    }

    /// Set the value of the atomic float to `value`.
    pub fn set(&self, value: bool) {
        self.atomic.store(value, Ordering::Relaxed)
    }
}

impl Default for AtomicBool {
    fn default() -> Self {
        AtomicBool::new(false)
    }
}
