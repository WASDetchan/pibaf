use std::sync::LazyLock;

pub(in crate::vk) static ENTRY: LazyLock<ash::Entry> = LazyLock::new(|| {
    // Safety: Entry::load() cannot actually cause UB
    let entry = unsafe { ash::Entry::load() }.expect("vulkan is not suppoted");
    log::info!("Loaded entry");
    entry
});
