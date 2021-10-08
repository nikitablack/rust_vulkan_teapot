use ash::vk;

use crate::vulkan::VulkanData;
use vulkan_base::VulkanBase;

pub fn present(
    vulkan_data: &VulkanData,
    vulkan_base: &VulkanBase,
    image_index: u32,
) -> Result<bool, String> {
    let semaphores = [vulkan_data.rendering_finished_semaphore];
    let swapchains = [vulkan_base.swapchain];
    let indices = [image_index];
    let present_info = vk::PresentInfoKHR::builder()
        .wait_semaphores(&semaphores)
        .swapchains(&swapchains)
        .image_indices(&indices)
        .build();

    unsafe {
        if let Err(err) = vulkan_base
            .swapchain_loader
            .queue_present(vulkan_base.queue, &present_info)
        {
            if err == vk::Result::SUBOPTIMAL_KHR || err == vk::Result::ERROR_OUT_OF_DATE_KHR {
                return Ok(false);
            } else {
                return Err(String::from("failed to present"));
            }
        }
    }

    Ok(true)
}
