use crate::vulkan;
use ash::version::DeviceV1_0;
use ash::vk;

fn get_image_index(
    data: &mut vulkan::TitleScreenVulkanData,
    vulkan_base_data: &vulkan_base::VulkanBaseData,
) -> Result<u32, String> {
    let swapchain_loader = vulkan_base_data.get_swapchain_loader_ref();

    match unsafe {
        swapchain_loader.acquire_next_image(
            vulkan_base_data.swapchain,
            u64::MAX,
            data.image_available_semaphore,
            vk::Fence::null(),
        )
    } {
        Ok((index, _)) => Ok(index),
        Err(_) => Err(String::from("failed to acquire next image")),
    }
}

fn get_command_buffer(
    data: &mut vulkan::TitleScreenVulkanData,
    device: &ash::Device,
) -> Result<vk::CommandBuffer, String> {
    debug_assert!(data.command_buffers.len() == (common::NUM_RESOURCES_IN_FLIGHT as usize));

    let command_buffer = data.command_buffers[data.resource_index as usize];

    unsafe {
        device.free_command_buffers(data.command_pool, &[command_buffer]);

        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(data.command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        let command_buffers = match device.allocate_command_buffers(&allocate_info) {
            Ok(buf) => buf,
            Err(_) => {
                return Err(format!(
                    "failed to allocate command buffer fot active resource index {}",
                    data.resource_index
                ))
            }
        };

        data.command_buffers[data.resource_index as usize] = command_buffers[0];
    }

    Ok(data.command_buffers[data.resource_index as usize])
}

fn begin_command_buffer(
    device: &ash::Device,
    command_buffer: vk::CommandBuffer,
) -> Result<(), String> {
    let begin_info =
        vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    unsafe {
        match device.begin_command_buffer(command_buffer, &begin_info) {
            Err(_) => return Err(String::from("failed to begin graphics command buffer")),
            _ => (),
        }
    }

    Ok(())
}

fn begin_render_pass(
    data: &mut vulkan::TitleScreenVulkanData,
    vulkan_base_data: &vulkan_base::VulkanBaseData,
    image_index: usize,
    command_buffer: vk::CommandBuffer,
) {
    let device = vulkan_base_data.get_device_ref();

    let clear_color = vk::ClearColorValue {
        float32: [0.5f32, 0.1f32, 0.1f32, 0.1f32],
    };
    let clear_values = vec![vk::ClearValue { color: clear_color }];

    let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
        .render_pass(data.render_pass)
        .framebuffer(data.framebuffers[image_index])
        .render_area(vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: vulkan_base_data.surface_extent,
        })
        .clear_values(&clear_values);

    unsafe {
        device.cmd_begin_render_pass(
            command_buffer,
            &render_pass_begin_info,
            vk::SubpassContents::INLINE,
        );
    }
}

fn push_constant(
    data: &mut vulkan::TitleScreenVulkanData,
    vulkan_base_data: &vulkan_base::VulkanBaseData,
    command_buffer: vk::CommandBuffer,
) {
    let device = vulkan_base_data.get_device_ref();

    let push_const = vec![
        2.0f32 / (vulkan_base_data.surface_extent.width as f32),
        2.0 / (vulkan_base_data.surface_extent.height as f32),
        -1.0,
        -1.0,
    ];
    let push_const_raw = common::utils::t_to_u8(&push_const);

    unsafe {
        device.cmd_push_constants(
            command_buffer,
            data.pipeline_layout,
            vk::ShaderStageFlags::VERTEX,
            0,
            push_const_raw,
        );
    }
}

fn set_viewport(vulkan_base_data: &vulkan_base::VulkanBaseData, command_buffer: vk::CommandBuffer) {
    let device = vulkan_base_data.get_device_ref();

    let viewport = vk::Viewport {
        x: 0.0,
        y: 0.0,
        width: vulkan_base_data.surface_extent.width as f32,
        height: vulkan_base_data.surface_extent.height as f32,
        min_depth: 0.0f32,
        max_depth: 1.0f32,
    };

    unsafe {
        device.cmd_set_viewport(command_buffer, 0, &[viewport]);
    }
}

fn draw_ui(
    vulkan_base_data: &vulkan_base::VulkanBaseData,
    ui_draw_data: &imgui::DrawData,
    command_buffer: vk::CommandBuffer,
) {
    let device = vulkan_base_data.get_device_ref();

    let mut global_vertex_offset = 0u32;
    let mut global_index_offset = 0u32;
    for draw_list in ui_draw_data.draw_lists() {
        for cmd in draw_list.commands() {
            match cmd {
                imgui::DrawCmd::Elements {
                    count,
                    cmd_params:
                        imgui::DrawCmdParams {
                            clip_rect,
                            vtx_offset,
                            idx_offset,
                            ..
                        },
                } => {
                    let scissor = vk::Rect2D {
                        offset: vk::Offset2D {
                            x: f32::max(0.0, clip_rect[0]).floor() as i32,
                            y: f32::max(0.0, clip_rect[1]).floor() as i32,
                        },
                        extent: vk::Extent2D {
                            width: (clip_rect[2] - clip_rect[0]).abs().ceil() as u32,
                            height: (clip_rect[3] - clip_rect[1]).abs().ceil() as u32,
                        },
                    };

                    unsafe {
                        device.cmd_set_scissor(command_buffer, 0, &[scissor]);

                        device.cmd_draw_indexed(
                            command_buffer,
                            count as u32,
                            1,
                            (idx_offset as u32) + global_index_offset,
                            ((vtx_offset as u32) + global_vertex_offset) as i32,
                            0,
                        );
                    }
                }

                _ => (),
            }
        }

        global_vertex_offset += draw_list.vtx_buffer().len() as u32;
        global_index_offset += draw_list.idx_buffer().len() as u32;
    }
}

fn submit(
    data: &mut vulkan::TitleScreenVulkanData,
    vulkan_base_data: &vulkan_base::VulkanBaseData,
    command_buffer: vk::CommandBuffer,
) -> Result<(), String> {
    let device = vulkan_base_data.get_device_ref();
    let device_data = vulkan_base_data
        .physical_devices
        .get(vulkan_base_data.selected_physical_device_index)
        .expect("physical device index is out of bounds");
    let resource_fence = data.fences[data.resource_index as usize];

    let wait_semaphores = [data.image_available_semaphore];
    let masks = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
    let cmd_buffers = [command_buffer];
    let signal_semaphores = [data.graphics_finished_semaphore];
    let submit_info = vk::SubmitInfo::builder()
        .wait_semaphores(&wait_semaphores)
        .wait_dst_stage_mask(&masks)
        .command_buffers(&cmd_buffers)
        .signal_semaphores(&signal_semaphores)
        .build();

    unsafe {
        match device.queue_submit(device_data.queue, &[submit_info], resource_fence) {
            Err(_) => Err(String::from("failed to submit graphics command buffer")),
            _ => Ok(()),
        }
    }
}

fn present(
    data: &mut vulkan::TitleScreenVulkanData,
    vulkan_base_data: &vulkan_base::VulkanBaseData,
    image_index: u32,
) -> Result<(), String> {
    let swapchain_loader = vulkan_base_data.get_swapchain_loader_ref();
    let device_data = vulkan_base_data
        .physical_devices
        .get(vulkan_base_data.selected_physical_device_index)
        .expect("physical device index is out of bounds");

    let semaphores = [data.graphics_finished_semaphore];
    let swapchains = [vulkan_base_data.swapchain];
    let indices = [image_index];
    let present_info = vk::PresentInfoKHR::builder()
        .wait_semaphores(&semaphores)
        .swapchains(&swapchains)
        .image_indices(&indices)
        .build();

    unsafe {
        if let Err(err) = swapchain_loader.queue_present(device_data.queue, &present_info) {
            if err == vk::Result::SUBOPTIMAL_KHR || err == vk::Result::ERROR_OUT_OF_DATE_KHR {
                panic!("swapchain resized");
            } else {
                return Err(String::from("failed to present"));
            }
        }
    }

    Ok(())
}

pub fn draw(
    data: &mut vulkan::TitleScreenVulkanData,
    vulkan_base_data: &vulkan_base::VulkanBaseData,
    ui_draw_data: &imgui::DrawData,
) -> common::VulkanResult {
    let device = vulkan_base_data.get_device_ref();
    let image_index = get_image_index(data, vulkan_base_data)?;
    let command_buffer = get_command_buffer(data, device)?;

    begin_command_buffer(device, command_buffer)?;
    begin_render_pass(data, vulkan_base_data, image_index as usize, command_buffer);
    push_constant(data, vulkan_base_data, command_buffer);
    set_viewport(vulkan_base_data, command_buffer);

    unsafe {
        device.cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            data.pipeline_layout,
            0,
            &[data.descriptor_sets[data.resource_index as usize]],
            &[],
        );
        device.cmd_bind_index_buffer(
            command_buffer,
            data.index_mem_buffers[data.resource_index as usize].buffer,
            0,
            vk::IndexType::UINT16,
        );
        device.cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            data.pipeline,
        );
    }

    draw_ui(vulkan_base_data, ui_draw_data, command_buffer);

    unsafe {
        device.cmd_end_render_pass(command_buffer);

        if let Err(_) = device.end_command_buffer(command_buffer) {
            return Err(format!(
                "failed to end command buffer fot active resource index {}",
                data.resource_index
            ));
        }
    }

    submit(data, vulkan_base_data, command_buffer)?;
    present(data, vulkan_base_data, image_index)?;

    Ok(())
}
