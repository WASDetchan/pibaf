use ash::vk;
use pibaf::vk::{
    extension, instance::{Instance, InstanceCreateInfo}, validation_layer::{self, *}
};

fn main() {
    env_logger::init();

    const REQUIRED_LAYERS: [ValidationLayer; 1] = [ValidationLayer::KhronosValidation];
    let available_layers = validation_layer::enumerate();
    let available_extension = extension::enumerate();

    dbg!(&available_layers, available_layers.len());
    dbg!(&available_extension, available_extension.len());

    let layers =
        AvailableValidationLayers::from_available_and_required(&available_layers, &REQUIRED_LAYERS)
            .expect("Failed to find KhronosValidation layer");

    // dbg!(&layers);

    let info = InstanceCreateInfo::builder()
        .api_version(vk::API_VERSION_1_0)
        .validation_layers(layers)
        .build()
        .unwrap();
    let _ = Instance::create_vk_instance(info);
}
