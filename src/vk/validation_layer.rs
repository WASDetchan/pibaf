use std::ffi::CStr;

pub struct AvailableValidationLayers;

impl AvailableValidationLayers {
    pub fn names(&self) -> Vec<&'static CStr> {
        Vec::new()
    }
}
