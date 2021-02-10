use crate::vulkan;
use ash::version::DeviceV1_0;
use ash::vk;

pub fn create_fences(
    data: &mut vulkan::TitleScreenVulkanData,
    vulkan_base_data: &vulkan_base::VulkanBaseData,
) -> common::VulkanResult {
    let device_ref = vulkan_base_data.get_device_ref();

    let create_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

    data.fences = Vec::with_capacity(common::NUM_RESOURCES_IN_FLIGHT as usize);

    for i in 0..common::NUM_RESOURCES_IN_FLIGHT {
        let fence = match unsafe { device_ref.create_fence(&create_info, None) } {
            Ok(fence) => fence,
            Err(_) => return Err(format!("failed to create fence {}", i)),
        };

        data.fences.push(fence);
    }

    if let Some(debug_utils) = vulkan_base_data.debug_utils_loader.as_ref() {
        for (i, &f) in data.fences.iter().enumerate() {
            common::set_debug_utils_object_name(
                debug_utils,
                device_ref.handle(),
                f,
                format!("title screen fence {}", i),
            );
        }
    }

    Ok(())
}
