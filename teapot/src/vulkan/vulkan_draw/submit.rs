use ash::vk;

use crate::vulkan::VulkanData;
use vulkan_base::VulkanBase;

pub fn submit(
    vulkan_data: &VulkanData,
    vulkan_base: &VulkanBase,
    command_buffer: vk::CommandBuffer,
) -> Result<(), String> {
    let fence = vulkan_data.fences[vulkan_data.curr_resource_index as usize];

    let wait_semaphores = [vulkan_data.image_available_semaphore];
    let masks = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
    let cmd_buffers = [command_buffer];
    let signal_semaphores = [vulkan_data.rendering_finished_semaphore];
    let submit_info = vk::SubmitInfo::builder()
        .wait_semaphores(&wait_semaphores)
        .wait_dst_stage_mask(&masks)
        .command_buffers(&cmd_buffers)
        .signal_semaphores(&signal_semaphores)
        .build();

    unsafe {
        vulkan_base
            .device
            .queue_submit(vulkan_base.queue, &[submit_info], fence)
            .map_err(|_| String::from("failed to submit graphics command buffer"))?
    }

    Ok(())
}
