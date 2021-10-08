use ash::vk;

use crate::vulkan::VulkanData;
use vulkan_base::VulkanBase;

pub enum GetImageIndexResult {
    Index(u32),
    ShouldRebuildSwapchain,
}

pub fn get_image_index(
    vulkan_data: &VulkanData,
    vulkan_base: &VulkanBase,
) -> Result<GetImageIndexResult, String> {
    let (index, is_suboptimal) = match unsafe {
        vulkan_base.swapchain_loader.acquire_next_image(
            vulkan_base.swapchain,
            u64::MAX,
            vulkan_data.image_available_semaphore,
            vk::Fence::null(),
        )
    } {
        Ok((index, is_suboptimal)) => (index, is_suboptimal),
        Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
            return Ok(GetImageIndexResult::ShouldRebuildSwapchain)
        }
        Err(_) => return Err(String::from("failed to acquire next image")),
    };

    if is_suboptimal {
        return Ok(GetImageIndexResult::ShouldRebuildSwapchain);
    }

    Ok(GetImageIndexResult::Index(index))
}
