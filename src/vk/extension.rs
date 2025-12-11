use std::ffi::CStr;

use ash::vk;
use strum::{EnumCount, IntoEnumIterator};

use super::{entry, error::expect_vk_success};

const EXTENSION_NAMES: [&CStr; Extension::COUNT] =
    [c"VK_KHR_surface", c"__UNKNOWN_EXTENSION", c"__UNREACHABLE_EXTENSION"];

/// Enumeration of all supported extensions, plus UnknownExtension and UnreachableExtension
#[derive(Clone, Copy, strum::EnumCount, strum::EnumIter, PartialEq, Eq, Debug)]
#[repr(usize)]
pub enum Extension {
    KhrSurface,
    UnknownExtension,
    UnreachableExtension,
}

impl Extension {
    /// Get the name of the extension
    pub fn name(&self) -> &'static CStr {
        // EXTENSION_NAMES.len() is always the number of enum variants, meaning its
        // discriminant is always in range
        EXTENSION_NAMES[*self as usize]
    }

    /// Return the first enum variant with name mathing the given string. Returns UnknownExtension if the name doesn't match any variant
    pub fn identify_name(name: &CStr) -> Self {
        Self::iter().find(|extension| extension.name() == name).unwrap_or(Self::UnknownExtension)
    }
}

/// Stores info about a extension. Guarantees extension's availability, meaning this struct can
/// only be obtained from enumerating the extensions
#[derive(Clone, Debug)]
pub struct AvailableExtension {
    extension: Extension,
    properties: vk::ExtensionProperties,
}

impl AvailableExtension {
    /// Returns the extension variant
    pub fn extension(&self) -> &Extension {
        &self.extension
    }

    /// Returns the extension's properties
    pub fn raw_properties(&self) -> &vk::ExtensionProperties {
        &self.properties
    }
}
/// Enumerates available instance extensions. Ignores unkwown names.
pub fn enumerate() -> Vec<AvailableExtension> {
    // Safety: ENTRY is never destroyed
    let extensions = expect_vk_success("Failed to enumerate extensions", unsafe {
        entry::ENTRY.enumerate_instance_extension_properties(None)
    });

    let extensions = extensions
        .into_iter()
        .flat_map(|prop| {
            let name = prop
                .extension_name_as_c_str()
                .expect("Got invalid extension name from enumeration");
            let extension = Extension::identify_name(name);
            Some(AvailableExtension {
                extension,
                properties: prop,
            })
        })
        .collect();
    log::trace!("Enumerated extensions, avalilable extensions: {extensions:#?}");
    extensions
}

/// List of some of the available extensions. Guarantees avalilability. Used to safely
/// enable those extensions withoutadditional checks
#[derive(Debug)]
pub struct AvailableExtensions {
    extensions: Vec<Extension>,
}

impl AvailableExtensions {
    /// Returns Vec of contained extensions' names
    pub fn names(&self) -> Vec<&'static CStr> {
        self.extensions.iter().map(|extension| extension.name()).collect()
    }

    /// Slice of avalilable extensions
    pub fn extensions(&self) -> &[Extension] {
        &self.extensions
    }

    /// Adds a extension to the available extension list
    pub fn add(&mut self, extension: AvailableExtension) {
        self.extensions.push(extension.extension);
    }

    /// If avalilable contains each element from required, returns Self containing all required
    /// extensions, else returns None
    pub fn from_available_and_required(
        available: &[AvailableExtension],
        required: &[Extension],
    ) -> Option<Self> {
        let mut has_requirements = true;
        required
            .iter()
            .for_each(|req| has_requirements &= available.iter().any(|avail| avail.extension == *req));

        has_requirements.then(|| Self {
            extensions: required.to_vec(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn name() {
        let extension = Extension::KhrSurface;
        assert_eq!(c"VK_KHR_surface", extension.name());
    }

    #[test]
    fn identify() {
        let extension = Extension::identify_name(c"VK_KHR_surface");
        assert_eq!(extension, Extension::KhrSurface);
    }

    #[test]
    fn identify_not_found() {
        let extension = Extension::identify_name(c"garbage");
        assert_eq!(extension, Extension::UnknownExtension);
    }

    #[test]
    fn has_khronos() {
        let available = enumerate();
        let required = [Extension::KhrSurface];

        let res =
            AvailableExtensions::from_available_and_required(&available, &required).unwrap();

        assert_eq!(res.extensions, &required);
    }

    #[test]
    fn does_not_have_unknown() {
        let available = enumerate();
        let required = [Extension::UnreachableExtension];

        let res = AvailableExtensions::from_available_and_required(&available, &required);

        assert!(res.is_none());
    }

    #[test]
    fn names() {
        let available = enumerate();
        let required = [Extension::KhrSurface];

        let res =
            AvailableExtensions::from_available_and_required(&available, &required).unwrap();

        assert_eq!(&res.names(), &[c"VK_KHR_surface"]);
    }
}
