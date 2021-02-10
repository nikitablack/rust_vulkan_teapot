use crate::vulkan;
use ash::version::DeviceV1_0;
use ash::vk;

pub fn create_command_pool(
    data: &mut vulkan::TitleScreenVulkanData,
    vulkan_base_data: &vulkan_base::VulkanBaseData,
) -> common::VulkanResult {
    let device = vulkan_base_data.get_device_ref();
    let ref device_data = vulkan_base_data
        .physical_devices
        .get(vulkan_base_data.selected_physical_device_index)
        .expect("physical device index is out of bounds");

    let create_info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
        .queue_family_index(device_data.queue_family);

    data.command_pool = match unsafe { device.create_command_pool(&create_info, None) } {
        Ok(pool) => pool,
        Err(_) => return Err(String::from("failed to create command pool")),
    };

    if let Some(debug_utils) = vulkan_base_data.debug_utils_loader.as_ref() {
        common::set_debug_utils_object_name(
            debug_utils,
            device.handle(),
            data.command_pool,
            String::from("command pool"),
        );
    }

    Ok(())
}
