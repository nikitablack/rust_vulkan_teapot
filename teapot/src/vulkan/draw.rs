use crate::VulkanData;
use ash::vk;
use cgmath::{num_traits::ToPrimitive, perspective, Deg, Matrix4, Point3, Vector3};
use vulkan_base::VulkanBase;

enum GetImageIndexResult {
    Index(u32),
    ShouldRebuildSwapchain,
}

fn get_image_index(
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

fn wait_resource_available(
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

fn reset_command_pool(
    vulkan_data: &mut VulkanData,
    vulkan_base: &VulkanBase,
) -> Result<(), String> {
    let command_pool = vulkan_data.command_pools[vulkan_data.curr_resource_index as usize];
    let available_command_buffers =
        &mut vulkan_data.available_command_buffers[vulkan_data.curr_resource_index as usize];
    let used_command_buffers =
        &mut vulkan_data.used_command_buffers[vulkan_data.curr_resource_index as usize];

    unsafe {
        let curr_resource_index = vulkan_data.curr_resource_index;

        vulkan_base
            .device
            .reset_command_pool(command_pool, vk::CommandPoolResetFlags::RELEASE_RESOURCES)
            .map_err(|_| {
                format!(
                    "failed to reset command pool for frame index {}",
                    curr_resource_index
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
    let command_pool = vulkan_data.command_pools[vulkan_data.curr_resource_index as usize];
    let available_command_buffers =
        &mut vulkan_data.available_command_buffers[vulkan_data.curr_resource_index as usize];

    if available_command_buffers.is_empty() {
        unsafe {
            let allocate_info = vk::CommandBufferAllocateInfo::builder()
                .command_pool(command_pool)
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(10)
                .build();

            let curr_resource_index = vulkan_data.curr_resource_index;

            let mut command_buffers = vulkan_base
                .device
                .allocate_command_buffers(&allocate_info)
                .map_err(|_| {
                    format!(
                        "failed to allocate command buffers for frame index {}",
                        curr_resource_index
                    )
                })?;

            available_command_buffers.append(&mut command_buffers);
        }
    }

    let command_buffer = available_command_buffers.pop().unwrap();

    let used_command_buffers =
        &mut vulkan_data.used_command_buffers[vulkan_data.curr_resource_index as usize];

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
        float32: [0.5f32, 0.5f32, 0.5f32, 1.0f32],
    };
    let clear_values = vec![vk::ClearValue { color: clear_color }];

    let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
        .render_pass(vulkan_data.render_pass)
        .framebuffer(vulkan_data.framebuffers[image_index])
        .render_area(vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: vulkan_base.surface_extent,
        })
        .clear_values(&clear_values)
        .build();

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

fn set_scissor(vulkan_base: &VulkanBase, command_buffer: vk::CommandBuffer) {
    let scissor = vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent: vk::Extent2D {
            width: vulkan_base.surface_extent.width,
            height: vulkan_base.surface_extent.height,
        },
    };

    unsafe {
        vulkan_base
            .device
            .cmd_set_scissor(command_buffer, 0, &[scissor]);
    }
}

fn reset_descriptor_pool(
    vulkan_data: &mut VulkanData,
    vulkan_base: &VulkanBase,
) -> Result<(), String> {
    let descriptor_pool = vulkan_data.descriptor_pools[vulkan_data.curr_resource_index as usize];

    unsafe {
        let curr_resource_index = vulkan_data.curr_resource_index;

        vulkan_base
            .device
            .reset_descriptor_pool(descriptor_pool, vk::DescriptorPoolResetFlags::empty())
            .map_err(|_| {
                format!(
                    "failed to reset descriptor pool for frame index {}",
                    curr_resource_index
                )
            })?;
    }

    Ok(())
}

fn allocate_descriptor_set(
    vulkan_data: &mut VulkanData,
    vulkan_base: &VulkanBase,
) -> Result<vk::DescriptorSet, String> {
    let layouts = [vulkan_data.descriptor_set_layout; 1];

    let alloc_info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(vulkan_data.descriptor_pools[vulkan_data.curr_resource_index as usize])
        .set_layouts(&layouts)
        .build();

    let descriptor_sets = match unsafe { vulkan_base.device.allocate_descriptor_sets(&alloc_info) }
    {
        Ok(sets) => sets,
        Err(_) => return Err(String::from("failed to allocate descriptor sets")),
    };

    let set = descriptor_sets[0];

    vulkan_utils::set_debug_utils_object_name(
        &vulkan_base.debug_utils_loader,
        vulkan_base.device.handle(),
        set,
        "descriptor set",
    );

    Ok(set)
}

fn update_descriptor_set(
    vulkan_data: &VulkanData,
    vulkan_base: &VulkanBase,
    set: vk::DescriptorSet,
) {
    let control_point_buffer_info = vk::DescriptorBufferInfo {
        buffer: vulkan_data.control_points_mem_buffer.buffer,
        offset: 0,
        range: vk::WHOLE_SIZE,
    };

    let instance_buffer_info = vk::DescriptorBufferInfo {
        buffer: vulkan_data.instances_mem_buffer.buffer,
        offset: 0,
        range: vk::WHOLE_SIZE,
    };

    let uniform_buffer_info = vk::DescriptorBufferInfo {
        buffer: vulkan_data.uniform_mem_buffers[vulkan_data.curr_resource_index as usize].buffer,
        offset: 0,
        range: vk::WHOLE_SIZE,
    };

    let infos_1 = [control_point_buffer_info];
    let write_descriptor_set_1 = vk::WriteDescriptorSet::builder()
        .dst_set(set)
        .dst_binding(0)
        .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
        .buffer_info(&infos_1)
        .build();

    let infos_2 = [instance_buffer_info];
    let write_descriptor_set_2 = vk::WriteDescriptorSet::builder()
        .dst_set(set)
        .dst_binding(1)
        .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
        .buffer_info(&infos_2)
        .build();

    let infos_3 = [uniform_buffer_info];
    let write_descriptor_set_3 = vk::WriteDescriptorSet::builder()
        .dst_set(set)
        .dst_binding(2)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .buffer_info(&infos_3)
        .build();

    unsafe {
        vulkan_base.device.update_descriptor_sets(
            &[
                write_descriptor_set_1,
                write_descriptor_set_2,
                write_descriptor_set_3,
            ],
            &[],
        );
    }
}

fn submit(
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

fn present(
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

pub fn draw(
    vulkan_data: &mut VulkanData,
    vulkan_base: &VulkanBase,
    time_since_biginning_sec: f32,
) -> Result<(), String> {
    let get_image_index_result = get_image_index(vulkan_data, vulkan_base)?;
    let image_index = match get_image_index_result {
        GetImageIndexResult::Index(index) => index,
        GetImageIndexResult::ShouldRebuildSwapchain => {
            println!("swapchain is suboptimal or out of date");
            vulkan_data.should_resize = true;
            return Ok(());
        }
    };
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
    set_scissor(vulkan_base, command_buffer);

    reset_descriptor_pool(vulkan_data, vulkan_base)?;
    let descriptor_set = allocate_descriptor_set(vulkan_data, vulkan_base)?;
    update_descriptor_set(vulkan_data, vulkan_base, descriptor_set);

    let model: Matrix4<f32> = Matrix4::from_translation(Vector3::new(0.0, 1.0, 0.0))
        * Matrix4::from_angle_x(Deg::<f32>(120.0))
        * Matrix4::from_angle_z(Deg::<f32>(time_since_biginning_sec * 20.0));
    let view = Matrix4::look_at_rh(
        Point3::<f32>::new(0.0, 0.0, -10.0),
        Point3::<f32>::new(0.0, 0.0, 0.0),
        Vector3::<f32>::new(0.0, 1.0, 0.0),
    );
    let projection = perspective(
        Deg::<f32>(45.0),
        vulkan_base
            .surface_extent
            .width
            .to_f32()
            .expect("failed to convert surface width to f32")
            / vulkan_base
                .surface_extent
                .height
                .to_f32()
                .expect("failed to convert surface width to f32"),
        0.1,
        100.0,
    );

    let mvp = projection * view * model;

    let curr_uniform_buffer =
        &mut vulkan_data.uniform_mem_buffers[vulkan_data.curr_resource_index as usize];

    let mvp_data = cgmath::conv::array4(mvp);
    let mvp_data_bytes = bytemuck::cast_slice(&mvp_data);

    curr_uniform_buffer.allocation.mapped_slice_mut().unwrap()[..16 * 4]
        .copy_from_slice(mvp_data_bytes);

    unsafe {
        vulkan_base.device.cmd_push_constants(
            command_buffer,
            vulkan_data.pipeline_layout,
            vk::ShaderStageFlags::TESSELLATION_CONTROL,
            0,
            bytemuck::cast_slice(&[vulkan_data.tesselation_level]),
        );

        vulkan_base.device.cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            vulkan_data.pipeline_layout,
            0,
            &[descriptor_set],
            &[],
        );

        let curr_pipeline = match vulkan_data.is_wireframe_mode {
            true => vulkan_data.wireframe_pipeline,
            false => vulkan_data.solid_pipeline,
        };

        vulkan_base.device.cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            curr_pipeline,
        );

        vulkan_base.device.cmd_bind_index_buffer(
            command_buffer,
            vulkan_data.patches_mem_buffer.buffer,
            0,
            vk::IndexType::UINT16,
        );

        vulkan_base.device.cmd_draw_indexed(
            command_buffer,
            vulkan_data.patch_point_count,
            1,
            0,
            0,
            0,
        );
    }

    unsafe {
        vulkan_base.device.cmd_end_render_pass(command_buffer);

        vulkan_base
            .device
            .end_command_buffer(command_buffer)
            .map_err(|_| String::from("failed to end command buffer"))?
    }

    submit(vulkan_data, vulkan_base, command_buffer)?;
    if !present(vulkan_data, vulkan_base, image_index)? {
        println!("swapchain is suboptimal or out of date");
        vulkan_data.should_resize = true;
        return Ok(());
    }

    Ok(())
}
