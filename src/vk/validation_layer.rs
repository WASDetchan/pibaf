use std::ffi::CStr;

use ash::vk;
use strum::{EnumCount, IntoEnumIterator};

use super::{entry, error::expect_vk_success};

const VALIDATION_LAYER_NAMES: [&CStr; ValidationLayer::COUNT] =
    [c"VK_LAYER_KHRONOS_validation", c"__UNKNOW_LAYER", c"__UNREACHABLE_LAYER"];

/// Enumeration of all supported validation layers, plus UnknownLayer and UnreachableLayer
#[derive(Clone, Copy, strum::EnumCount, strum::EnumIter, PartialEq, Eq, Debug)]
#[repr(usize)]
pub enum ValidationLayer {
    KhronosValidation,
    UnknownLayer,
    UnreachableLayer,
}

impl ValidationLayer {
    /// Get the name of the validation layer
    pub fn name(&self) -> &'static CStr {
        // VALIDATION_LAYER_NAMES.len() is always the number of enum variants, meaning its
        // discriminant is always in range
        VALIDATION_LAYER_NAMES[*self as usize]
    }

    /// Return the first enum variant with name mathing the given string. Returns ValidationLayer::UnknownLayer if the name doesn't match any variant
    pub fn identify_name(name: &CStr) -> Self {
        Self::iter().find(|layer| layer.name() == name).unwrap_or(ValidationLayer::UnknownLayer)
    }
}

/// Stores info about a validation layer. Guarantees layer's availability, meaning this struct can
/// only be obtained from enumerating the layers
#[derive(Clone, Debug)]
pub struct AvailableValidationLayer {
    layer: ValidationLayer,
    properties: vk::LayerProperties,
}

impl AvailableValidationLayer {
    /// Returns the layer variant
    pub fn layer(&self) -> &ValidationLayer {
        &self.layer
    }

    /// Returns the layer's properties
    pub fn raw_properties(&self) -> &vk::LayerProperties {
        &self.properties
    }
}
/// Enumerates available instance validation layers. Ignores unkwown names.
pub fn enumerate() -> Vec<AvailableValidationLayer> {
    // Safety: ENTRY is never destroyed
    let layers = expect_vk_success("Failed to enumerate validation layers", unsafe {
        entry::ENTRY.enumerate_instance_layer_properties()
    });

    let layers = layers
        .into_iter()
        .flat_map(|prop| {
            let name = prop
                .layer_name_as_c_str()
                .expect("Got invalid layer name from enumeration");
            let layer = ValidationLayer::identify_name(name);
            Some(AvailableValidationLayer {
                layer,
                properties: prop,
            })
        })
        .collect();
    log::trace!("Enumerated validation layers, avalilable layers: {layers:#?}");
    layers
}

/// List of some of the available validation layers. Guarantees avalilability. Used to safely
/// enable those layers without additional checks
#[derive(Debug)]
pub struct AvailableValidationLayers {
    layers: Vec<ValidationLayer>,
}

impl AvailableValidationLayers {
    /// Returns Vec of contained layers' names
    pub fn names(&self) -> Vec<&'static CStr> {
        self.layers.iter().map(|layer| layer.name()).collect()
    }

    /// Slice of avalilable validation layers
    pub fn layers(&self) -> &[ValidationLayer] {
        &self.layers
    }

    /// Adds a layer to the available layer list
    pub fn add(&mut self, layer: AvailableValidationLayer) {
        self.layers.push(layer.layer);
    }

    /// If avalilable contains each element from required, returns Self containing all required
    /// layers, else returns None
    pub fn from_available_and_required(
        available: &[AvailableValidationLayer],
        required: &[ValidationLayer],
    ) -> Option<Self> {
        let mut has_requirements = true;
        required
            .iter()
            .for_each(|req| has_requirements &= available.iter().any(|avail| avail.layer == *req));

        has_requirements.then(|| Self {
            layers: required.to_vec(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn name() {
        let layer = ValidationLayer::KhronosValidation;
        assert_eq!(c"VK_LAYER_KHRONOS_validation", layer.name());
    }

    #[test]
    fn identify() {
        let layer = ValidationLayer::identify_name(c"VK_LAYER_KHRONOS_validation");
        assert_eq!(layer, ValidationLayer::KhronosValidation);
    }

    #[test]
    fn identify_not_found() {
        let layer = ValidationLayer::identify_name(c"garbage");
        assert_eq!(layer, ValidationLayer::UnknownLayer);
    }

    #[test]
    fn has_khronos() {
        let available = enumerate();
        let required = [ValidationLayer::KhronosValidation];

        let res =
            AvailableValidationLayers::from_available_and_required(&available, &required).unwrap();

        assert_eq!(res.layers, &required);
    }

    #[test]
    fn does_not_have_unknown() {
        let available = enumerate();
        let required = [ValidationLayer::UnreachableLayer];

        let res = AvailableValidationLayers::from_available_and_required(&available, &required);

        assert!(res.is_none());
    }

    #[test]
    fn names() {
        let available = enumerate();
        let required = [ValidationLayer::KhronosValidation];

        let res =
            AvailableValidationLayers::from_available_and_required(&available, &required).unwrap();

        assert_eq!(&res.names(), &[c"VK_LAYER_KHRONOS_validation"]);
    }
}
