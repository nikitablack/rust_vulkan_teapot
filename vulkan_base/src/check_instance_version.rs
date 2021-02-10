use ash::vk;

pub fn check_instance_version(vulkan_data: &crate::VulkanBaseData) -> crate::VulkanInitResult {
    let api_version = match vulkan_data.get_entry_ref().try_enumerate_instance_version() {
        Ok(result) => match result {
            Some(version) => version,
            None => vk::make_version(1, 0, 0),
        },
        Err(_) => {
            return Err(String::from("failed to enumerate instance version"));
        }
    };

    log::info!(
        "instance version: {}.{}.{}",
        vk::version_major(api_version),
        vk::version_minor(api_version),
        vk::version_patch(api_version)
    );

    if vk::version_major(api_version) < 1 && vk::version_minor(api_version) < 1 {
        return Err(String::from(
            "minimum supported vulkan api version is 1.1.0",
        ));
    }

    Ok(())
}
