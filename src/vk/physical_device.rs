//!
//! Utilities for safe vulkan physical device information querying
//!

use std::fmt::Debug;

use ash::vk;

use crate::vk::{Instance, error::expect_vk_success};

/// Properties of an available queue family. Guarantees that the queue family is available on the
/// stored device
#[derive(Debug)]
pub struct AvailableQueue {
    device: vk::PhysicalDevice,
    idx: usize,
    flags: vk::QueueFlags,
    // TODO: add the rest of the properties
}

impl AvailableQueue {
    /// Checks if the queue has the graphics bit
    pub fn has_graphics(&self) -> bool {
        self.flags.contains(vk::QueueFlags::GRAPHICS)
    }

    /// Checks if the queue belongs to the given physical device
    pub fn belongs_to_device(&self, device: &PhysicalDevice) -> bool {
        device.device == self.device
    }

    /// Get the index of the queue family
    pub fn get_family_idx(&self) -> usize {
        self.idx
    }

    fn from_family_prop(prop: vk::QueueFamilyProperties) -> Self {}
}

/// A handle to a vk::PhysicalDevice. Can only be acquired from enumerating physical devices,
/// guaranteeing that the device is available
pub struct PhysicalDevice {
    instance: Instance,
    device: vk::PhysicalDevice,
}

impl Debug for PhysicalDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PhysicalDevice {:?} of {:?}", self.device, self.instance)
    }
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
            self.instance
                .get_raw_ref()
                .get_physical_device_properties(self.device)
        }
    }

    /// Query PhysicalDeviceFeatures
    pub fn raw_features(&self) -> vk::PhysicalDeviceFeatures {
        // Safety: instance is not destroyed, a valid PhysicalDevice is passed
        unsafe {
            self.instance
                .get_raw_ref()
                .get_physical_device_features(self.device)
        }
    }

    /// Query QueueFamilyProperties
    pub fn raw_queue_family_properties(&self) -> Vec<vk::QueueFamilyProperties> {
        // Safety: instance is not destroyed, a valid PhysicalDevice is passed
        unsafe {
            self.instance
                .get_raw_ref()
                .get_physical_device_queue_family_properties(self.device)
        }
    }

    /// Get a vec of avalilable queue families.
    pub fn get_available_queues(&self) -> Vec<AvailableQueue> {
        self.raw_queue_family_properties()
            .into_iter()
            .enumerate()
            .flat_map(|(idx, prop)| AvailableQueueFamily {
                device: self.device,
                idx,
                flags: prop.queue_flags,
                queue_count: prop.queue_count,
            })
            .collect()
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
        .map(|dev| PhysicalDevice {
            device: dev,
            instance: instance.clone(),
        })
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
        let instance_info = InstanceCreateInfo::builder()
            .api_version(vk::API_VERSION_1_0)
            .build()
            .unwrap();

        let instance = Instance::create_vk_instance(instance_info);

        let devices = enumerate(&instance);

        assert!(!devices.is_empty());

        // Output of following functions cannot be verified, but it must be ensured that they do
        // not panic or fault
        let _ = devices[0].raw_device();
        let _ = devices[0].raw_properties();
        let _ = devices[0].raw_features();
    }

    #[test]
    fn debug_fmt() {
        let instance_info = InstanceCreateInfo::builder()
            .api_version(vk::API_VERSION_1_0)
            .build()
            .unwrap();

        let instance = Instance::create_vk_instance(instance_info);

        let devices = enumerate(&instance);

        assert!(!devices.is_empty());

        assert!(format!("{:?}", devices[0]).contains("PhysicalDevice"));
    }

    #[test]
    fn has_graphics_queue() {
        let instance_info = InstanceCreateInfo::builder()
            .api_version(vk::API_VERSION_1_0)
            .build()
            .unwrap();

        let instance = Instance::create_vk_instance(instance_info);

        let devices = enumerate(&instance);

        assert!(!devices.is_empty());

        let graphic_families = devices[0]
            .get_available_queue_families()
            .into_iter()
            .enumerate()
            .filter(|(_idx, qf)| qf.has_graphics())
            .collect::<Vec<_>>();

        assert!(graphic_families.len() > 0);
        assert!(graphic_families[0].1.has_graphics());
        assert!(graphic_families[0].1.queue_count() > 0);
        assert!(graphic_families[0].1.belongs_to_device(&devices[0]));
        assert_eq!(graphic_families[0].1.get_idx(), graphic_families[0].0);
    }
}
