use crate::vulkan;
use ash::version::DeviceV1_0;
use ash::vk;

pub fn create_descriptor_pools(
    vulkan_base: &vulkan_base::VulkanBase,
) -> Result<Vec<vk::DescriptorPool>, String> {
    let pool_size_1 = vk::DescriptorPoolSize {
        ty: vk::DescriptorType::STORAGE_BUFFER,
        descriptor_count: 100,
    };

    let pool_size_2 = vk::DescriptorPoolSize {
        ty: vk::DescriptorType::UNIFORM_BUFFER,
        descriptor_count: 100,
    };

    let sizes = [pool_size_1, pool_size_2];
    let create_info = vk::DescriptorPoolCreateInfo::builder()
        .max_sets(100)
        .pool_sizes(&sizes)
        .build();

    let mut descriptor_pools = Vec::with_capacity(crate::CONCURRENT_RESOURCE_COUNT as usize);

    for i in 0..crate::CONCURRENT_RESOURCE_COUNT {
        let pool = unsafe {
            vulkan_base
                .device
                .create_descriptor_pool(&create_info, None)
                .map_err(|_| {
                    for &p in &descriptor_pools {
                        vulkan_base.device.destroy_descriptor_pool(p, None);
                    }
                    format!("failed to create descriptor pool {}", i)
                })?
        };

        vulkan::set_debug_utils_object_name(
            &vulkan_base.debug_utils_loader,
            vulkan_base.device.handle(),
            pool,
            &format!("descriptor pool {}", i),
        );

        descriptor_pools.push(pool);
    }

    Ok(descriptor_pools)
}
