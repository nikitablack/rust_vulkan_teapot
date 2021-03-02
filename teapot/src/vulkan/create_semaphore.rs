use crate::vulkan;
use ash::version::DeviceV1_0;
use ash::vk;

pub fn create_semaphore(
    vulkan_base: &vulkan_base::VulkanBase,
    object_name: &str,
) -> Result<vk::Semaphore, String> {
    let create_info = vk::SemaphoreCreateInfo::default();

    let semaphore = unsafe {
        vulkan_base
            .device
            .create_semaphore(&create_info, None)
            .map_err(|_| format!("failed to create {}", object_name))?
    };

    vulkan::set_debug_utils_object_name(
        &vulkan_base.debug_utils_loader,
        vulkan_base.device.handle(),
        semaphore,
        object_name,
    );

    Ok(semaphore)
}
