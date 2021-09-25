use ash::extensions::ext;

pub fn create_debug_utils_loader(
    enable_debug_utils: bool,
    entry: &ash::Entry,
    instance: &ash::Instance,
) -> Option<ext::DebugUtils> {
    match enable_debug_utils {
        true => {
            log::info!("debug utils loader created");
            Some(ext::DebugUtils::new(&entry, &instance))
        }
        false => None,
    }
}
