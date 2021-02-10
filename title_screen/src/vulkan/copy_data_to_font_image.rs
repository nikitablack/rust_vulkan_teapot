use crate::vulkan;
use ash::version::DeviceV1_0;
use ash::vk;

fn allocate_command_buffer(
    device: &ash::Device,
    command_pool: vk::CommandPool,
) -> Result<vk::CommandBuffer, String> {
    let allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(1);

    let command_buffers = match unsafe { device.allocate_command_buffers(&allocate_info) } {
        Ok(buf) => buf,
        Err(_) => return Err(String::from("failed to allocate command buffer")),
    };

    Ok(command_buffers[0])
}

fn copy_buffer_to_image(
    device: &ash::Device,
    copy_queue: vk::Queue,
    command_buffer: vk::CommandBuffer,
    src_mem_buffer: &common::MemBuffer,
    dst_mem_image: &common::MemImage,
    dst_image_access_mask: vk::AccessFlags,
    dst_image_layout: vk::ImageLayout,
    dst_image_stage_mask: vk::PipelineStageFlags,
) -> Result<(), String> {
    let begin_info =
        vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    if let Err(_) = unsafe { device.begin_command_buffer(command_buffer, &begin_info) } {
        return Err(String::from(
            "failed to begin copy buffer to image command buffer",
        ));
    }

    let pre_copy_barrier = vk::ImageMemoryBarrier::builder()
        .image(dst_mem_image.image)
        .src_access_mask(vk::AccessFlags::empty())
        .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
        .old_layout(vk::ImageLayout::UNDEFINED)
        .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        })
        .build();

    unsafe {
        device.cmd_pipeline_barrier(
            command_buffer,
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::TRANSFER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[pre_copy_barrier],
        );
    }

    let copy_region = vk::BufferImageCopy {
        buffer_offset: 0,
        buffer_row_length: 0,
        buffer_image_height: 0,
        image_subresource: vk::ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            mip_level: 0,
            base_array_layer: 0,
            layer_count: 1,
        },
        image_offset: vk::Offset3D { x: 0, y: 0, z: 0 },
        image_extent: dst_mem_image.extent,
    };

    unsafe {
        device.cmd_copy_buffer_to_image(
            command_buffer,
            src_mem_buffer.buffer,
            dst_mem_image.image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &[copy_region],
        );
    }

    let post_copy_barrier = vk::ImageMemoryBarrier::builder()
        .image(dst_mem_image.image)
        .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
        .dst_access_mask(dst_image_access_mask)
        .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
        .new_layout(dst_image_layout)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        })
        .build();

    unsafe {
        device.cmd_pipeline_barrier(
            command_buffer,
            vk::PipelineStageFlags::TRANSFER,
            dst_image_stage_mask,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[post_copy_barrier],
        );
    }

    if let Err(_) = unsafe { device.end_command_buffer(command_buffer) } {
        return Err(String::from(
            "failed to end copy buffer to image command buffer",
        ));
    }

    let cmd_buffers = [command_buffer];
    let submit_info = vk::SubmitInfo::builder()
        .command_buffers(&cmd_buffers)
        .build();

    match unsafe { device.queue_submit(copy_queue, &[submit_info], vk::Fence::null()) } {
        Err(_) => return Err(String::from("failed to submit graphics command buffer")),
        _ => (),
    }

    if let Err(_) = unsafe { device.queue_wait_idle(copy_queue) } {
        return Err(String::from(
            "failed to wait queue idle on copy buffer to image",
        ));
    }

    Ok(())
}

pub fn copy_data_to_image(
    data: &mut vulkan::TitleScreenVulkanData,
    vulkan_base_data: &mut vulkan_base::VulkanBaseData,
    image_data: &[u8],
) -> common::VulkanResult {
    let device_ref = vulkan_base_data.get_device_ref();
    let device_data = vulkan_base_data
        .physical_devices
        .get(vulkan_base_data.selected_physical_device_index)
        .expect("physical device index is out of bounds");

    let staging_mem_buffer = match common::create_buffer(
        vulkan_base_data.get_allocator_ref(),
        image_data.len() as vk::DeviceSize,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk_mem::MemoryUsage::CpuOnly,
        vk_mem::AllocationCreateFlags::MAPPED,
    ) {
        Ok(buf) => buf,
        Err(_) => {
            return Err(String::from(
                "failed to create staging buffer for font image copy",
            ))
        }
    };

    unsafe {
        std::ptr::copy_nonoverlapping(
            image_data.as_ptr(),
            staging_mem_buffer
                .get_allocation_info_ref()
                .get_mapped_data(),
            image_data.len(),
        )
    };

    let command_buffer = match allocate_command_buffer(device_ref, data.command_pool) {
        Ok(cb) => cb,
        Err(msg) => return Err(msg),
    };

    if let Err(msg) = copy_buffer_to_image(
        device_ref,
        device_data.queue,
        command_buffer,
        &staging_mem_buffer,
        &data.font_mem_image,
        vk::AccessFlags::SHADER_READ,
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        vk::PipelineStageFlags::FRAGMENT_SHADER,
    ) {
        return Err(msg);
    }

    let _ = vulkan_base_data
        .get_allocator_mut()
        .destroy_buffer(staging_mem_buffer.buffer, &staging_mem_buffer.allocation);

    Ok(())
}
