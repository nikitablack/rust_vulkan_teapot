use ash::extensions::ext;

pub fn create_debug_utils_loader(entry: &ash::Entry, instance: &ash::Instance) -> ext::DebugUtils {
    let debug_utils_loader = ext::DebugUtils::new(&entry, &instance);

    log::info!("debug utils loader created");

    debug_utils_loader
}
