use ash::vk;

use crate::vulkan::VulkanData;
use vulkan_base::VulkanBase;

pub fn reset_descriptor_pool(
    vulkan_data: &mut VulkanData,
    vulkan_base: &VulkanBase,
) -> Result<(), String> {
    let descriptor_pool = vulkan_data.descriptor_pools[vulkan_data.curr_resource_index as usize];

    unsafe {
        let curr_resource_index = vulkan_data.curr_resource_index;

        vulkan_base
            .device
            .reset_descriptor_pool(descriptor_pool, vk::DescriptorPoolResetFlags::empty())
            .map_err(|_| {
                format!(
                    "failed to reset descriptor pool for frame index {}",
                    curr_resource_index
                )
            })?;
    }

    Ok(())
}
