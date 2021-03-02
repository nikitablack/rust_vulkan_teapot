use crate::vulkan;
use ash::version::DeviceV1_0;
use ash::vk;

pub fn create_fences(vulkan_base: &vulkan_base::VulkanBase) -> Result<Vec<vk::Fence>, String> {
    let create_info = vk::FenceCreateInfo::builder()
        .flags(vk::FenceCreateFlags::SIGNALED)
        .build();

    let mut fences = Vec::with_capacity(crate::CONCURRENT_RESOURCE_COUNT as usize);

    for i in 0..crate::CONCURRENT_RESOURCE_COUNT {
        let fence = unsafe {
            vulkan_base
                .device
                .create_fence(&create_info, None)
                .map_err(|_| {
                    for &f in &fences {
                        vulkan_base.device.destroy_fence(f, None);
                    }

                    format!("failed to create fence {}", i)
                })?
        };

        fences.push(fence);

        vulkan::set_debug_utils_object_name(
            &vulkan_base.debug_utils_loader,
            vulkan_base.device.handle(),
            fence,
            &format!("fence {}", i),
        );
    }

    Ok(fences)
}
