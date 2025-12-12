use std::ffi::{CStr, CString};

use strum::{EnumCount, IntoEnumIterator};

use super::{entry, error::expect_vk_success};

const EXTENSION_NAMES: [&CStr; Extension::COUNT] = [
    c"VK_KHR_surface",
    c"__UNKNOWN_EXTENSION",
    c"__UNREACHABLE_EXTENSION",
];

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
        Self::iter()
            .find(|extension| extension.name() == name)
            .unwrap_or(Self::UnknownExtension)
    }
}

/// Stores info about a extension. Guarantees extension's availability, meaning this struct can
/// only be obtained from enumerating the extensions
#[derive(Clone, Debug)]
pub struct AvailableExtension {
    extension: Extension,
    name: CString,
    spec_version: u32,
}

impl AvailableExtension {
    /// Returns the extension variant
    pub fn extension(&self) -> Extension {
        self.extension
    }

    /// Extension name
    pub fn name(&self) -> &CStr {
        &self.name
    }
    /// Extension spec version
    pub fn spec_version(&self) -> u32 {
        self.spec_version
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
                .expect("Got invalid extension name from enumeration")
                .to_owned();
            let extension = Extension::identify_name(&name);
            Some(AvailableExtension {
                extension,
                name,
                spec_version: prop.spec_version,
            })
        })
        .collect();
    log::trace!("Enumerated extensions, avalilable extensions: {extensions:#?}");
    extensions
}

/// List of some of the available extensions. Guarantees avalilability. Used to safely
/// enable those extensions withoutadditional checks
#[derive(Debug, Default)]
pub struct AvailableExtensions {
    extensions: Vec<AvailableExtension>,
}

impl AvailableExtensions {
    /// Returns Vec of contained extensions' names
    pub fn names(&self) -> Vec<&CStr> {
        self.extensions
            .iter()
            .map(|extension| extension.name())
            .collect()
    }

    /// Slice of avalilable extensions
    pub fn extensions(&self) -> &[AvailableExtension] {
        &self.extensions
    }

    /// Adds a extension to the available extension list
    pub fn add(&mut self, extension: AvailableExtension) {
        self.extensions.push(extension);
    }

    /// If avalilable contains each element from required, returns Self containing all required
    /// extensions, else returns None
    pub fn from_available_and_required(
        available: &[AvailableExtension],
        required: &[Extension],
    ) -> Option<Self> {
        let mut selected_extensions = Vec::with_capacity(required.len());
        for req in required {
            let ext = available.iter().find(|avail| avail.extension == *req)?;
            selected_extensions.push(ext);
        }

        Some(Self {
            extensions: selected_extensions.into_iter().cloned().collect(),
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

        let res = AvailableExtensions::from_available_and_required(&available, &required).unwrap();

        assert_eq!(res.extensions().len(), 1);
        assert_eq!(res.extensions()[0].extension(), Extension::KhrSurface);
        assert_ne!(res.extensions()[0].spec_version(), 0); 
    }

    #[test]
    fn manual_add() {
        let available = enumerate();

        let khr = available.into_iter().find(|e| e.extension() == Extension::KhrSurface).expect("KhrSurface extension not found");

        let mut res = AvailableExtensions::default();
        res.add(khr);

        assert_eq!(res.names(), &[c"VK_KHR_surface"]);
    }

    #[test]
    fn does_not_have_unreachable() {
        let available = enumerate();
        let required = [Extension::UnreachableExtension];

        let res = AvailableExtensions::from_available_and_required(&available, &required);

        assert!(res.is_none());
    }

    #[test]
    fn names() {
        let available = enumerate();
        let required = [Extension::KhrSurface];

        let res = AvailableExtensions::from_available_and_required(&available, &required).unwrap();

        assert_eq!(&res.names(), &[c"VK_KHR_surface"]);
    }
}
