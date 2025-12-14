use ash::vk;
use pibaf::vk::{
    extension::{self, AvailableExtensions, Extension},
    instance::{Instance, InstanceCreateInfo},
    physical_device,
    validation_layer::{self, *},
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

    let devices = physical_device::enumerate(&instance);
    let graphic_families = devices[0]
        .get_available_queue_families()
        .into_iter()
        .enumerate()
        .filter(|(_idx, qf)| qf.has_graphics())
        .collect::<Vec<_>>();

    log::info!("{:?}", graphic_families);

    assert!(graphic_families.len() > 0);
    assert!(graphic_families[0].1.has_graphics());
    assert!(graphic_families[0].1.queue_count() > 0);
    assert!(graphic_families[0].1.belongs_to_device(&devices[0]));
    assert_eq!(graphic_families[0].1.get_idx(), graphic_families[0].0);
}
