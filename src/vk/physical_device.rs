//!
//! Utilities for safe vulkan physical device information querying
//!

use ash::vk;

use crate::vk::{ Instance, error::expect_vk_success};

/// A handle to a vk::PhysicalDevice. Can only be acquired from enumerating physical devices,
/// guaranteeing that the device is available
#[derive(Debug)]
pub struct PhysicalDevice {
    instance: Instance,
    device: vk::PhysicalDevice,
}

impl PhysicalDevice {
    /// Get the inner PhysicalDevice
    pub fn raw_device(&self) -> vk::PhysicalDevice {
        self.device
    }

    /// Query PhysicalDeviceProperties
    pub fn raw_properties(&self) -> vk::PhysicalDeviceProperties {
        // Safety: instance is not destroyed, a valid PhysicalDevice is passed
        unsafe {
            self.instance.get_raw_ref().get_physical_device_properties(self.device)
        }
    }

    /// Query PhysicalDeviceFeatures
    pub fn raw_features(&self) -> vk::PhysicalDeviceFeatures {
        // Safety: instance is not destroyed, a valid PhysicalDevice is passed
        unsafe {
            self.instance.get_raw_ref().get_physical_device_features(self.device)
        }
    }
}
/// Enumerate avalilable vulkan physical devices
pub fn enumerate(instance: &Instance) -> Vec<PhysicalDevice> {
    // Safety: instacne is not destroyed
    let devices = expect_vk_success("Failed to enumerate_physical_devices", unsafe {
        instance.get_raw_ref().enumerate_physical_devices()
    });


    let devices = devices
        .into_iter()
        .map(|dev| PhysicalDevice { device: dev, instance: instance.clone() })
        .collect();
        log::trace!("Enumerated physical devices, avalilable devices: {devices:#?}");
    devices

}

#[cfg(test)]
mod test {
    use super::*;
    use crate::vk::instance::InstanceCreateInfo;

    #[test]
    fn device_is_found() {
        let instance_info = InstanceCreateInfo::builder().api_version(vk::API_VERSION_1_0).build().unwrap();

        let instance = Instance::create_vk_instance (instance_info);

        let devices = enumerate(&instance);

        assert!(!devices.is_empty());
    }
}
