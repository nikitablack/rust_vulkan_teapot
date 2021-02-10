use crate::vulkan;
use ash::version::DeviceV1_0;

pub fn wait_resource_available(
    data: &mut vulkan::TitleScreenVulkanData,
    vulkan_base_data: &vulkan_base::VulkanBaseData,
) -> common::VulkanResult {
    let device = vulkan_base_data.get_device_ref();

    let resource_fence = data.fences[data.resource_index as usize];

    unsafe {
        if let Err(_) = device.wait_for_fences(&[resource_fence], true, u64::MAX) {
            return Err(format!(
                "failed to wait for resource fence {}",
                data.resource_index
            ));
        }

        if let Err(_) = device.reset_fences(&[resource_fence]) {
            return Err(format!(
                "failed to reset resource fence {}",
                data.resource_index
            ));
        }
    }

    Ok(())
}
