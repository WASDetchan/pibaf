use std::sync::LazyLock;

pub(in crate::vk) static ENTRY: LazyLock<ash::Entry> = LazyLock::new(|| {
    // Safety: Entry::load() cannot actually cause UB
    unsafe { ash::Entry::load() }.expect("vulkan is not suppoted")
});
