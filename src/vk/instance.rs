use std::{
    ffi::{CStr, CString, NulError, c_char},
    ops::Deref,
};

use ash::{
    self,
    vk::{self, Handle},
};

use crate::{
    arc_array::UnsafeArcArray,
    vk::{
        entry, error::expect_vk_success, extension::AvailableExtensions,
        validation_layer::AvailableValidationLayers,
    },
};

/// ash::Instance wrapper that destroys the Instance when dropped
pub struct RawInstance(ash::Instance);

impl Drop for RawInstance {
    fn drop(&mut self) {
        let handle = self.0.handle().as_raw();
        unsafe {
            self.0.destroy_instance(None);
        }
        log::info!("Destroyed instance: {handle}");
    }
}

impl RawInstance {
    /// # Safety
    /// The ash::Instance should not be destroyed
    pub unsafe fn get_raw_ref(&self) -> &ash::Instance {
        &self.0
    }
}

const MAX_INSTANCES: usize = 1;
static RAW_INSTANCES: UnsafeArcArray<MAX_INSTANCES, RawInstance> = UnsafeArcArray::new();

/// A handle to a RawInstance
pub struct Instance {
    id: usize,
}

impl Drop for Instance {
    fn drop(&mut self) {
        // Safety: Instance's existence guarantees that the RawInstance is valid
        unsafe {
            RAW_INSTANCES.dec_count(self.id);
        }
    }
}

impl Deref for Instance {
    type Target = RawInstance;
    fn deref(&self) -> &Self::Target {
        // Safety: Instance's existence guarantees that the RawInstance under its index is initialized
        unsafe { RAW_INSTANCES.get_ref(self.id) }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InstanceCreateError {
    #[error("Vulkan could not be loaded")]
    VulkanLoadFailure(#[from] ash::LoadingError),
}

impl Instance {
    /// # Safety
    /// The ash::Instance should not be destroyed  
    /// # Panics
    /// Panics if the instance limit is reached
    pub unsafe fn from_raw(raw_instance: ash::Instance) -> Self {
        Self {
            id: RAW_INSTANCES
                .acquire_and_init(|| RawInstance(raw_instance))
                .expect("Failed to initialize instance (no free space)"),
        }
    }

    /// Creates a vulkan instance
    /// # Panics
    /// Panics if vulkan is not supported
    pub fn create_vk_instance(info: InstanceCreateInfo) -> Self {
        log::trace!("Creating Instance: {info:#?}" );
        let create_info = info.create_raw();

        // Safety: InstanceCreateInfo guarantees that it gives valid create_info
        let instance = expect_vk_success("Failed to create vk::Instance", unsafe {
            entry::ENTRY.create_instance(&create_info.vk_instance_create_info(), None)
        });

        log::info!("Cretated instance, handle: {}", instance.handle().as_raw());

        // Safety: The only reference to this instance is being put into the array
        unsafe { Self::from_raw(instance) }
    }
}

impl Clone for Instance {
    fn clone(&self) -> Self {
        RAW_INSTANCES.inc_count(self.id);
        Self { id: self.id }
    }
}

/// Struct containing pointers to data required to create vk::Instance. This intermediate struct is
/// needed because of double inderection of the data
pub struct RawInstanceCreateInfo<'a> {
    enabled_validation_layers: Vec<*const c_char>, // 'a lifetime referencing InstanceCreateInfo
    enabled_extension: Vec<*const c_char>,         // 'a lifetime  referencing InstanceCreateInfo
    application_info: vk::ApplicationInfo<'a>,
    owned_info: &'a InstanceCreateInfo,
}

impl RawInstanceCreateInfo<'_> {
    /// Creates the actual vk::InstanceCreateInfo from self's data pointers
    pub fn vk_instance_create_info(&self) -> vk::InstanceCreateInfo<'_> {
        vk::InstanceCreateInfo::default()
            .flags(self.owned_info.flags)
            .enabled_layer_names(&self.enabled_validation_layers)
            .enabled_extension_names(&self.enabled_extension)
            .application_info(&self.application_info)
    }
}

/// Owned data for vk::InstanceCreateInfo
#[derive(Debug)]
pub struct InstanceCreateInfo {
    enabled_validation_layers: Vec<&'static CStr>,
    enabled_extensions: Vec<&'static CStr>,

    flags: vk::InstanceCreateFlags,

    application_name: CString,
    application_version: u32,

    engine_name: CString,
    engine_version: u32,

    api_version: u32,
}

#[bon::bon]
impl InstanceCreateInfo {
    /// Creates InstanceCreateInfo. Fails if any of the given strings contain nulls
    #[builder]
    pub fn new(
        validation_layers: Option<AvailableValidationLayers>,
        extensions: Option<AvailableExtensions>,
        enumerate_portability: Option<bool>,
        application_name: Option<&[u8]>,
        application_version: Option<u32>,
        engine_name: Option<&[u8]>,
        engine_version: Option<u32>,
        api_version: u32,
    ) -> Result<Self, NulError> {
        let application_name = if let Some(name) = application_name {
            CString::new(name)?
        } else {
            CString::from(c"")
        };

        let engine_name = if let Some(name) = engine_name {
            CString::new(name)?
        } else {
            CString::from(c"")
        };

        let enabled_validation_layers = if let Some(layers) = validation_layers {
            layers.names()
        } else {
            Vec::new()
        };

        let enabled_extensions = extensions
            .as_ref()
            .map_or_else(Vec::new, AvailableExtensions::names);

        let mut flags = vk::InstanceCreateFlags::empty();
        if enumerate_portability.is_some_and(|c| c) {
            flags |= vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
        }

        let application_version = application_version.unwrap_or(0);
        let engine_version = engine_version.unwrap_or(0);

        Ok(Self {
            enabled_validation_layers,
            enabled_extensions,
            flags,
            application_name,
            application_version,
            engine_name,
            engine_version,
            api_version,
        })
    }

    /// Creates the intermediate RawInstanceCreateInfo struct that stores the pointers to this struct
    pub fn create_raw(&self) -> RawInstanceCreateInfo<'_> {
        let extension_name_ptrs = self
            .enabled_extensions
            .iter()
            .map(|&s: &&CStr| s.as_ptr())
            .collect::<Vec<_>>();

        let validation_layer_name_ptrs = self
            .enabled_validation_layers
            .iter()
            .map(|&s: &&CStr| s.as_ptr())
            .collect::<Vec<_>>();

        let application_info = vk::ApplicationInfo::default()
            .application_name(self.application_name.as_c_str())
            .application_version(self.application_version)
            .engine_name(self.engine_name.as_c_str())
            .engine_version(self.engine_version)
            .api_version(self.api_version);

        RawInstanceCreateInfo {
            enabled_validation_layers: validation_layer_name_ptrs,
            enabled_extension: extension_name_ptrs,
            application_info,
            owned_info: self,
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    #[test]
    fn empty_validation_and_extension() {
        let info = InstanceCreateInfo::builder()
            .engine_name(b"pibaf")
            .engine_version(1)
            .application_name(b"test")
            .api_version(vk::API_VERSION_1_0)
            .build()
            .unwrap();
        let _ = Instance::create_vk_instance(info);
    }

    #[test]
    fn khronos_validation() {
        use crate::vk::validation_layer::{self, *};
        const REQUIRED_LAYERS: [ValidationLayer; 1] = [ValidationLayer::KhronosValidation];
        let available_layers = validation_layer::enumerate();

        let layers = AvailableValidationLayers::from_available_and_required(
            &available_layers,
            &REQUIRED_LAYERS,
        )
        .expect("Failed to find KhronosValidation layer");

        let info = InstanceCreateInfo::builder()
            .api_version(vk::API_VERSION_1_0)
            .validation_layers(layers)
            .build()
            .unwrap();
        let _ = Instance::create_vk_instance(info);
    }

    #[test]
    fn khr_surface() {
        use crate::vk::extension::{self, *};
        const REQUIRED_EXTENSIONS: [Extension; 1] = [Extension::KhrSurface];
        let available_extensions = extension::enumerate();

        let extensions = AvailableExtensions::from_available_and_required(
            &available_extensions,
            &REQUIRED_EXTENSIONS,
        )
        .expect("Failed to find KhrSurface extension");

        let info = InstanceCreateInfo::builder()
            .api_version(vk::API_VERSION_1_0)
            .extensions(extensions)
            .build()
            .unwrap();
        let _ = Instance::create_vk_instance(info);
    }

    #[test]
    fn extension_and_layer() {

        use crate::vk::validation_layer::{self, *};
        const REQUIRED_LAYERS: [ValidationLayer; 1] = [ValidationLayer::KhronosValidation];
        let available_layers = validation_layer::enumerate();

        let layers = AvailableValidationLayers::from_available_and_required(
            &available_layers,
            &REQUIRED_LAYERS,
        )
        .expect("Failed to find KhronosValidation layer");


        use crate::vk::extension::{self, *};
        const REQUIRED_EXTENSIONS: [Extension; 1] = [Extension::KhrSurface];
        let available_extensions = extension::enumerate();

        let extensions = AvailableExtensions::from_available_and_required(
            &available_extensions,
            &REQUIRED_EXTENSIONS,
        )
        .expect("Failed to find KhrSurface extension");

        let info = InstanceCreateInfo::builder()
            .api_version(vk::API_VERSION_1_0)
            .extensions(extensions)
            .validation_layers(layers)
            .build()
            .unwrap();
        let _ = Instance::create_vk_instance(info);
    }
}
