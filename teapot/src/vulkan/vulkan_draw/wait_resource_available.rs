use crate::vulkan::VulkanData;
use vulkan_base::VulkanBase;

pub fn wait_resource_available(
    vulkan_data: &VulkanData,
    vulkan_base: &VulkanBase,
) -> Result<(), String> {
    let fence = vulkan_data.fences[vulkan_data.curr_resource_index as usize];

    unsafe {
        vulkan_base
            .device
            .wait_for_fences(&[fence], true, u64::MAX)
            .map_err(|_| {
                format!(
                    "failed to wait for resource fence {}",
                    vulkan_data.curr_resource_index
                )
            })?;

        vulkan_base.device.reset_fences(&[fence]).map_err(|_| {
            format!(
                "failed to reset resource fence {}",
                vulkan_data.curr_resource_index
            )
        })?;
    }

    Ok(())
}
