use ash::vk;

/// Enumeration of all possible VkResult codes
#[cfg_attr(test, derive(strum::EnumIter))] // for testing all enum variants
#[derive(PartialEq, Debug, strum::FromRepr, strum::Display)]
#[repr(i32)]
pub enum VulkanResult {
    #[doc = "Command completed successfully"]
    #[strum(to_string = "")]
    Success = 0,
    #[doc = "A fence or query has not yet completed"]
    #[strum(to_string = "NOT_READY")]
    NotReady = 1,
    #[doc = "A wait operation has not completed in the specified time"]
    #[strum(to_string = "TIMEOUT")]
    Timeout = 2,
    #[doc = "An event is signaled"]
    #[strum(to_string = "EVENT_SET")]
    EventSet = 3,
    #[doc = "An event is unsignaled"]
    #[strum(to_string = "EVENT_RESET")]
    EventReset = 4,
    #[doc = "A return array was too small for the result"]
    #[strum(to_string = "INCOMPLETE")]
    Incomplete = 5,
    #[doc = "A host memory allocation has failed"]
    #[strum(to_string = "ERROR_OUT_OF_HOST_MEMORY")]
    ErrorOutOfHostMemory = -1,
    #[doc = "A device memory allocation has failed"]
    #[strum(to_string = "ERROR_OUT_OF_DEVICE_MEMORY")]
    ErrorOutOfDeviceMemory = -2,
    #[doc = "Initialization of an object has failed"]
    #[strum(to_string = "ERROR_INITIALIZATION_FAILED")]
    ErrorInitializationFailed = -3,
    #[doc = "The logical device has been lost. See <https://registry.khronos.org/vulkan/specs/1.3-extensions/html/vkspec.html#devsandqueues-lost-device>"]
    #[strum(to_string = "ERROR_DEVICE_LOST")]
    ErrorDeviceLost = -4,
    #[doc = "Mapping of a memory object has failed"]
    #[strum(to_string = "ERROR_MEMORY_MAP_FAILED")]
    ErrorMemoryMapFailed = -5,
    #[doc = "Layer specified does not exist"]
    #[strum(to_string = "ERROR_LAYER_NOT_PRESENT")]
    ErrorLayerNotPresent = -6,
    #[doc = "Extension specified does not exist"]
    #[strum(to_string = "ERROR_EXTENSION_NOT_PRESENT")]
    ErrorExtensionNotPresent = -7,
    #[doc = "Requested feature is not available on this device"]
    #[strum(to_string = "ERROR_FEATURE_NOT_PRESENT")]
    ErrorFeatureNotPresent = -8,
    #[doc = "Unable to find a Vulkan driver"]
    #[strum(to_string = "ERROR_INCOMPATIBLE_DRIVER")]
    ErrorIncompatibleDriver = -9,
    #[doc = "Too many objects of the type have already been created"]
    #[strum(to_string = "ERROR_TOO_MANY_OBJECTS")]
    ErrorTooManyObjects = -10,
    #[doc = "Requested format is not supported on this device"]
    #[strum(to_string = "ERROR_FORMAT_NOT_SUPPORTED")]
    ErrorFormatNotSupported = -11,
    #[doc = "A requested pool allocation has failed due to fragmentation of the pool's memory"]
    #[strum(to_string = "ERROR_FRAGMENTED_POOL")]
    ErrorFragmentedPool = -12,
    #[doc = "An unknown error has occurred, due to an implementation or application bug"]
    #[strum(to_string = "ERROR_UNKNOWN")]
    ErrorUnknown = -13,
}

impl VulkanResult {
    fn doc(&self) -> &'static str {
        match self {
            Self::Success => "Command completed successfully",
            Self::NotReady => "A fence or query has not yet completed",
            Self::Timeout => "A wait operation has not completed in the specified time",
            Self::EventSet => "An event is signaled",
            Self::EventReset => "An event is unsignaled",
            Self::Incomplete => "A return array was too small for the result",
            Self::ErrorOutOfHostMemory => "A host memory allocation has failed",
            Self::ErrorOutOfDeviceMemory => "A device memory allocation has failed",
            Self::ErrorInitializationFailed => "Initialization of an object has failed",
            Self::ErrorDeviceLost => {
                "The logical device has been lost. See <https://registry.khronos.org/vulkan/specs/1.3-extensions/html/vkspec.html#devsandqueues-lost-device>"
            }
            Self::ErrorMemoryMapFailed => "Mapping of a memory object has failed",
            Self::ErrorLayerNotPresent => "Layer specified does not exist",
            Self::ErrorExtensionNotPresent => "Extension specified does not exist",
            Self::ErrorFeatureNotPresent => "Requested feature is not available on this device",
            Self::ErrorIncompatibleDriver => "Unable to find a Vulkan driver",
            Self::ErrorTooManyObjects => "Too many objects of the type have already been created",
            Self::ErrorFormatNotSupported => "Requested format is not supported on this device",
            Self::ErrorFragmentedPool => {
                "A requested pool allocation has failed due to fragmentation of the pool's memory"
            }
            Self::ErrorUnknown => {
                "An unknown error has occurred, due to an implementation or application bug"
            }
        }
    }
}

impl From<vk::Result> for VulkanResult {
    fn from(value: vk::Result) -> Self {
        Self::from_repr(value.as_raw()).expect("all VkResult cases are covered")
    }
}

pub fn fatal_vk_error<T: Into<VulkanResult>>(msg: &str, error: T) -> ! {
    let e = error.into();
    log::error!("fatal: {}: {} ({})", msg, e, e.doc());
    panic!("fatal: {}: {} ({})", msg, e, e.doc());
}

pub fn expect_vk_success<T>(msg: &str, res: Result<T, vk::Result>) -> T {
    match res {
        Ok(i) => i,
        Err(e) => fatal_vk_error(msg, e),
    }
}

#[cfg(test)]
mod test {
    use strum::IntoEnumIterator;

    use super::*;

    #[test]
    fn from_vk_result() {
        let result = vk::Result::from_raw(-13);
        let vulkan_result = VulkanResult::from(result);
        assert_eq!(vulkan_result, VulkanResult::ErrorUnknown);
    }

    #[test]
    fn name() {
        let vulkan_result = VulkanResult::from(vk::Result::from_raw(-13));
        let name = vulkan_result.to_string();
        assert_eq!(name, "ERROR_UNKNOWN");
    }

    #[test]
    fn doc() {
        let vulkan_result = VulkanResult::from(vk::Result::from_raw(-13));
        let doc = vulkan_result.doc();
        assert_eq!(
            doc,
            "An unknown error has occurred, due to an implementation or application bug"
        )
    }

    #[test]
    fn doc_all() {
        let _ : Vec<_> = VulkanResult::iter().map(|res| res.doc()).collect();
    }

    #[test]
    #[should_panic]
    fn fatal() {
        let vulkan_result = VulkanResult::from(vk::Result::from_raw(-13));
        fatal_vk_error("ohno", vulkan_result)
    }

    #[test]
    #[should_panic]
    fn fatal_from_vk_result() {
        let result = vk::Result::from_raw(-13);
        fatal_vk_error("ohno", result)
    }

    #[test]
    #[should_panic]
    fn expect_err_vk_result() {
        let result = Err(vk::Result::from_raw(-13));
        expect_vk_success("ohno", result)
    }

    #[test]
    fn expect_ok_vk_result() {
        let result = Ok(11);
        assert_eq!(expect_vk_success("ohno", result), 11);
    }

}
