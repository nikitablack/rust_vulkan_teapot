use ash::vk;

use crate::vulkan::VulkanData;
use vulkan_base::VulkanBase;

pub fn reset_command_pool(
    vulkan_data: &mut VulkanData,
    vulkan_base: &VulkanBase,
) -> Result<(), String> {
    let command_pool = vulkan_data.command_pools[vulkan_data.curr_resource_index as usize];
    let available_command_buffers =
        &mut vulkan_data.available_command_buffers[vulkan_data.curr_resource_index as usize];
    let used_command_buffers =
        &mut vulkan_data.used_command_buffers[vulkan_data.curr_resource_index as usize];

    unsafe {
        let curr_resource_index = vulkan_data.curr_resource_index;

        vulkan_base
            .device
            .reset_command_pool(command_pool, vk::CommandPoolResetFlags::RELEASE_RESOURCES)
            .map_err(|_| {
                format!(
                    "failed to reset command pool for frame index {}",
                    curr_resource_index
                )
            })?;

        available_command_buffers.append(used_command_buffers);
    }

    Ok(())
}
