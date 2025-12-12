use ash::vk;
use pibaf::vk::{
    extension::{self, AvailableExtensions, Extension}, instance::{Instance, InstanceCreateInfo}, physical_device, validation_layer::{self, *}
};

fn main() {
    env_logger::init();

    const REQUIRED_LAYERS: [ValidationLayer; 1] = [ValidationLayer::KhronosValidation];
    let available_layers = validation_layer::enumerate();
    let available_extension = extension::enumerate();

    let layers =
        AvailableValidationLayers::from_available_and_required(&available_layers, &REQUIRED_LAYERS)
            .expect("Failed to find KhronosValidation layer");

    let extensions = AvailableExtensions::from_available_and_required(
        &available_extension,
        &[Extension::KhrSurface],
    )
    .expect("Failed to find KhrSurface extension");
    // dbg!(&layers);

    let info = InstanceCreateInfo::builder()
        .api_version(vk::API_VERSION_1_0)
        .validation_layers(layers)
        .extensions(extensions)
        .build()
        .unwrap();
    let instance = Instance::create_vk_instance(info);

    let physical_devices = physical_device::enumerate(&instance);
    _ = physical_devices;
}
