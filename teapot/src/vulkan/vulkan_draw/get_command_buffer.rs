use ash::vk;

use crate::vulkan::VulkanData;
use vulkan_base::VulkanBase;

pub fn get_command_buffer(
    vulkan_data: &mut VulkanData,
    vulkan_base: &VulkanBase,
) -> Result<vk::CommandBuffer, String> {
    let command_pool = vulkan_data.command_pools[vulkan_data.curr_resource_index as usize];
    let available_command_buffers =
        &mut vulkan_data.available_command_buffers[vulkan_data.curr_resource_index as usize];

    if available_command_buffers.is_empty() {
        unsafe {
            let allocate_info = vk::CommandBufferAllocateInfo::builder()
                .command_pool(command_pool)
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(10)
                .build();

            let curr_resource_index = vulkan_data.curr_resource_index;

            let mut command_buffers = vulkan_base
                .device
                .allocate_command_buffers(&allocate_info)
                .map_err(|_| {
                    format!(
                        "failed to allocate command buffers for frame index {}",
                        curr_resource_index
                    )
                })?;

            available_command_buffers.append(&mut command_buffers);
        }
    }

    let command_buffer = available_command_buffers.pop().unwrap();

    let used_command_buffers =
        &mut vulkan_data.used_command_buffers[vulkan_data.curr_resource_index as usize];

    used_command_buffers.push(command_buffer);

    Ok(command_buffer)
}
