use ash::vk;

pub fn create_semaphore(
    device: &ash::Device,
    debug_utils_loader: &ash::extensions::ext::DebugUtils,
    object_name: &str,
) -> Result<vk::Semaphore, String> {
    log::info!("{}: creating", object_name);

    let create_info = vk::SemaphoreCreateInfo::default();

    let semaphore = unsafe {
        device
            .create_semaphore(&create_info, None)
            .map_err(|_| format!("failed to create {}", object_name))?
    };

    crate::set_debug_utils_object_name(debug_utils_loader, device.handle(), semaphore, object_name);

    log::info!("{}: created", object_name);

    Ok(semaphore)
}
