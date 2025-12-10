use std::ffi::CStr;

pub struct AvailableExtensions;

impl AvailableExtensions {
    pub fn names(&self) -> Vec<&'static CStr> {
        Vec::new()
    }
}
