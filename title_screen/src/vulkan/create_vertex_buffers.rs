use crate::vulkan;
use ash::vk;

pub fn create_vertex_buffer(
    vulkan_base_data: &vulkan_base::VulkanBaseData,
    size: vk::DeviceSize,
    name: String,
) -> Result<common::MemBuffer, String> {
    log::info!("creating buffer {} with size {}", name, size);

    let device = vulkan_base_data.get_device_ref();

    let mem_buffer = match common::create_buffer(
        vulkan_base_data.get_allocator_ref(),
        size,
        vk::BufferUsageFlags::STORAGE_BUFFER,
        vk_mem::MemoryUsage::CpuToGpu,
        vk_mem::AllocationCreateFlags::MAPPED,
    ) {
        Ok(buf) => buf,
        Err(_) => return Err(format!("failed to allocate buffer {}", name.clone())),
    };

    if let Some(debug_utils) = vulkan_base_data.debug_utils_loader.as_ref() {
        common::set_debug_utils_object_name(
            debug_utils,
            device.handle(),
            mem_buffer.buffer,
            name.clone(),
        );

        common::set_debug_utils_object_name(
            debug_utils,
            device.handle(),
            mem_buffer.get_allocation_info_ref().get_device_memory(),
            name,
        );
    }

    Ok(mem_buffer)
}

pub fn create_vertex_buffers(
    data: &mut vulkan::TitleScreenVulkanData,
    vulkan_base_data: &vulkan_base::VulkanBaseData,
) -> common::VulkanResult {
    debug_assert!(data.vertex_mem_buffers.is_empty());

    data.vertex_mem_buffers
        .reserve(common::NUM_RESOURCES_IN_FLIGHT as usize);

    for i in 0..common::NUM_RESOURCES_IN_FLIGHT {
        let mem_buffer = create_vertex_buffer(
            vulkan_base_data,
            100,
            format!("title screen vertex buffer {}", i),
        )?;

        data.vertex_mem_buffers.push(mem_buffer);
    }

    Ok(())
}
