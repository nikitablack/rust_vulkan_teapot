use ash::vk;

pub fn create_fences(
    device: &ash::Device,
    debug_utils_loader: &ash::extensions::ext::DebugUtils,
) -> Result<Vec<vk::Fence>, String> {
    log::info!("creating fences");

    let create_info = vk::FenceCreateInfo::builder()
        .flags(vk::FenceCreateFlags::SIGNALED)
        .build();

    let mut fences = Vec::with_capacity(crate::CONCURRENT_RESOURCE_COUNT as usize);

    for i in 0..crate::CONCURRENT_RESOURCE_COUNT {
        let fence = unsafe {
            device.create_fence(&create_info, None).map_err(|_| {
                for &f in &fences {
                    device.destroy_fence(f, None);
                }

                format!("failed to create fence {}", i)
            })?
        };

        fences.push(fence);

        vulkan_utils::set_debug_utils_object_name(
            debug_utils_loader,
            device.handle(),
            fence,
            &format!("fence {}", i),
        );
    }

    log::info!("fences created");

    Ok(fences)
}
