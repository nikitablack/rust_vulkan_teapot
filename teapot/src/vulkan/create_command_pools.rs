use ash::vk;

pub fn create_command_pools(
    device: &ash::Device,
    queue_family: u32,
    debug_utils_loader: &ash::extensions::ext::DebugUtils,
) -> Result<Vec<vk::CommandPool>, String> {
    log::info!("creating command pools");

    let create_info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::TRANSIENT)
        .queue_family_index(queue_family);

    let mut command_pools = Vec::with_capacity(crate::CONCURRENT_RESOURCE_COUNT as usize);

    for i in 0..crate::CONCURRENT_RESOURCE_COUNT {
        let command_pool = unsafe {
            device
                .create_command_pool(&create_info, None)
                .map_err(|_| {
                    for &cp in &command_pools {
                        device.destroy_command_pool(cp, None);
                    }

                    format!("failed to create command pool {}", i)
                })?
        };

        command_pools.push(command_pool);

        vulkan_utils::set_debug_utils_object_name(
            debug_utils_loader,
            device.handle(),
            command_pool,
            &format!("command pool {}", i),
        );
    }

    log::info!("command pools created");

    Ok(command_pools)
}
