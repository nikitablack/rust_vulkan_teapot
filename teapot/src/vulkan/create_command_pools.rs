use crate::vulkan;
use ash::version::DeviceV1_0;
use ash::vk;

pub fn create_command_pools(
    vulkan_base: &vulkan_base::VulkanBase,
) -> Result<Vec<vk::CommandPool>, String> {
    let create_info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::TRANSIENT)
        .queue_family_index(vulkan_base.queue_family);

    let mut command_pools = Vec::with_capacity(crate::CONCURRENT_RESOURCE_COUNT as usize);

    for i in 0..crate::CONCURRENT_RESOURCE_COUNT {
        let command_pool = unsafe {
            vulkan_base
                .device
                .create_command_pool(&create_info, None)
                .map_err(|_| {
                    for &cp in &command_pools {
                        vulkan_base.device.destroy_command_pool(cp, None);
                    }

                    format!("failed to create command pool {}", i)
                })?
        };

        command_pools.push(command_pool);

        vulkan::set_debug_utils_object_name(
            &vulkan_base.debug_utils_loader,
            vulkan_base.device.handle(),
            command_pool,
            &format!("command pool {}", i),
        );
    }

    Ok(command_pools)
}
