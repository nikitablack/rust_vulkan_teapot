use crate::VulkanData;
use ash::version::DeviceV1_0;
use ash::vk;
use vulkan_base::VulkanBase;

fn get_image_index(vulkan_data: &VulkanData, vulkan_base: &VulkanBase) -> Result<u32, String> {
    let (index, _) = unsafe {
        vulkan_base
            .swapchain_loader
            .acquire_next_image(
                vulkan_base.swapchain,
                u64::MAX,
                vulkan_data.image_available_semaphore,
                vk::Fence::null(),
            )
            .map_err(|_| String::from("failed to acquire next image"))?
    };

    Ok(index)
}

fn wait_resource_available(
    vulkan_data: &VulkanData,
    vulkan_base: &VulkanBase,
) -> Result<(), String> {
    let fence = vulkan_data.fences[vulkan_data.frame_index as usize];

    unsafe {
        vulkan_base
            .device
            .wait_for_fences(&[fence], true, u64::MAX)
            .map_err(|_| {
                format!(
                    "failed to wait for resource fence {}",
                    vulkan_data.frame_index
                )
            })?;

        vulkan_base
            .device
            .reset_fences(&[fence])
            .map_err(|_| format!("failed to reset resource fence {}", vulkan_data.frame_index))?;
    }

    Ok(())
}

fn reset_command_pool(
    vulkan_data: &mut VulkanData,
    vulkan_base: &VulkanBase,
) -> Result<(), String> {
    let command_pool = vulkan_data.command_pools[vulkan_data.frame_index as usize];
    let available_command_buffers =
        &mut vulkan_data.available_command_buffers[vulkan_data.frame_index as usize];
    let used_command_buffers =
        &mut vulkan_data.used_command_buffers[vulkan_data.frame_index as usize];

    unsafe {
        let frame_index = vulkan_data.frame_index;

        vulkan_base
            .device
            .reset_command_pool(command_pool, vk::CommandPoolResetFlags::RELEASE_RESOURCES)
            .map_err(|_| {
                format!(
                    "failed to reset command pool for frame index {}",
                    frame_index
                )
            })?;

        available_command_buffers.append(used_command_buffers);
    }

    Ok(())
}

fn get_command_buffer(
    vulkan_data: &mut VulkanData,
    vulkan_base: &VulkanBase,
) -> Result<vk::CommandBuffer, String> {
    let command_pool = vulkan_data.command_pools[vulkan_data.frame_index as usize];
    let available_command_buffers =
        &mut vulkan_data.available_command_buffers[vulkan_data.frame_index as usize];

    if available_command_buffers.is_empty() {
        unsafe {
            let allocate_info = vk::CommandBufferAllocateInfo::builder()
                .command_pool(command_pool)
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(10)
                .build();

            let frame_index = vulkan_data.frame_index;

            let mut command_buffers = vulkan_base
                .device
                .allocate_command_buffers(&allocate_info)
                .map_err(|_| {
                    format!(
                        "failed to allocate command buffers for frame index {}",
                        frame_index
                    )
                })?;

            available_command_buffers.append(&mut command_buffers);
        }
    }

    let command_buffer = available_command_buffers.pop().unwrap();

    let used_command_buffers =
        &mut vulkan_data.used_command_buffers[vulkan_data.frame_index as usize];

    used_command_buffers.push(command_buffer);

    Ok(command_buffer)
}

fn begin_command_buffer(
    vulkan_base: &VulkanBase,
    command_buffer: vk::CommandBuffer,
) -> Result<(), String> {
    let begin_info = vk::CommandBufferBeginInfo::builder()
        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
        .build();

    unsafe {
        vulkan_base
            .device
            .begin_command_buffer(command_buffer, &begin_info)
            .map_err(|_| String::from("failed to begin command buffer"))?;
    }

    Ok(())
}

fn begin_render_pass(
    vulkan_data: &VulkanData,
    vulkan_base: &VulkanBase,
    image_index: usize,
    command_buffer: vk::CommandBuffer,
) {
    let clear_color = vk::ClearColorValue {
        float32: [0.5f32, 0.1f32, 0.1f32, 0.1f32],
    };
    let clear_values = vec![vk::ClearValue { color: clear_color }];

    let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
        .render_pass(vulkan_data.render_pass)
        .framebuffer(vulkan_data.framebuffers[image_index])
        .render_area(vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: vulkan_base.surface_extent,
        })
        .clear_values(&clear_values);

    unsafe {
        vulkan_base.device.cmd_begin_render_pass(
            command_buffer,
            &render_pass_begin_info,
            vk::SubpassContents::INLINE,
        );
    }
}

fn set_viewport(vulkan_base: &VulkanBase, command_buffer: vk::CommandBuffer) {
    let viewport = vk::Viewport {
        x: 0.0,
        y: 0.0,
        width: vulkan_base.surface_extent.width as f32,
        height: vulkan_base.surface_extent.height as f32,
        min_depth: 0.0f32,
        max_depth: 1.0f32,
    };

    unsafe {
        vulkan_base
            .device
            .cmd_set_viewport(command_buffer, 0, &[viewport]);
    }
}

fn submit(
    vulkan_data: &VulkanData,
    vulkan_base: &VulkanBase,
    command_buffer: vk::CommandBuffer,
) -> Result<(), String> {
    let fence = vulkan_data.fences[vulkan_data.frame_index as usize];

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

fn present(
    vulkan_data: &VulkanData,
    vulkan_base: &VulkanBase,
    image_index: u32,
) -> Result<(), String> {
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
                println!("swapchain is suboptimal or out of date");
            } else {
                return Err(String::from("failed to present"));
            }
        }
    }

    Ok(())
}

pub fn draw(vulkan_data: &mut VulkanData, vulkan_base: &VulkanBase) -> Result<(), String> {
    let image_index = get_image_index(vulkan_data, vulkan_base)?;
    wait_resource_available(vulkan_data, vulkan_base)?;
    reset_command_pool(vulkan_data, vulkan_base)?;
    let command_buffer = get_command_buffer(vulkan_data, vulkan_base)?;
    begin_command_buffer(vulkan_base, command_buffer)?;
    begin_render_pass(
        vulkan_data,
        vulkan_base,
        image_index as usize,
        command_buffer,
    );
    set_viewport(vulkan_base, command_buffer);

    unsafe {
        vulkan_base.device.cmd_end_render_pass(command_buffer);

        vulkan_base
            .device
            .end_command_buffer(command_buffer)
            .map_err(|_| String::from("failed to end command buffer"))?
    }

    submit(vulkan_data, vulkan_base, command_buffer)?;
    present(vulkan_data, vulkan_base, image_index)?;

    Ok(())
}
