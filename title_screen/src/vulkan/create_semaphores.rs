use crate::vulkan;
use ash::version::DeviceV1_0;
use ash::vk;

pub fn create_semaphores(
    data: &mut vulkan::TitleScreenVulkanData,
    vulkan_base_data: &vulkan_base::VulkanBaseData,
) -> common::VulkanResult {
    let device = vulkan_base_data.get_device_ref();

    let create_info = vk::SemaphoreCreateInfo {
        ..Default::default()
    };

    data.image_available_semaphore = match unsafe { device.create_semaphore(&create_info, None) } {
        Ok(semaphore) => semaphore,
        Err(_) => return Err(String::from("failed to create image available semaphore")),
    };

    if let Some(debug_utils) = vulkan_base_data.debug_utils_loader.as_ref() {
        common::set_debug_utils_object_name(
            debug_utils,
            device.handle(),
            data.image_available_semaphore,
            String::from("title screen image available semaphore"),
        );
    }

    data.graphics_finished_semaphore = match unsafe { device.create_semaphore(&create_info, None) }
    {
        Ok(semaphore) => semaphore,
        Err(_) => return Err(String::from("failed to create graphics finished semaphore")),
    };

    if let Some(debug_utils) = vulkan_base_data.debug_utils_loader.as_ref() {
        common::set_debug_utils_object_name(
            debug_utils,
            device.handle(),
            data.graphics_finished_semaphore,
            String::from("title screen graphics finished semaphore"),
        );
    }

    Ok(())
}
