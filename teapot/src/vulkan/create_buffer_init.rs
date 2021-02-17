use crate::vulkan;
use ash::version::DeviceV1_0;
use ash::vk;
use vulkan_base::VulkanBase;

#[derive(Default)]
struct InternalState {
    staging_mem_buffer: Option<vulkan::MemBuffer>,
    gpu_mem_buffer: Option<vulkan::MemBuffer>,
    command_pool: Option<vk::CommandPool>,
}

pub fn create_buffer_init(
    vulkan_base: &VulkanBase,
    init_data: &[u8],
    buffer_usage: vk::BufferUsageFlags,
    buffer_access_mask: vk::AccessFlags,
    buffer_stage_flags: vk::PipelineStageFlags,
    object_name: &str,
) -> Result<vulkan::MemBuffer, String> {
    let mut internal_state = InternalState::default();

    let res = create_buffer_init_internal(
        &mut internal_state,
        vulkan_base,
        init_data,
        buffer_usage,
        buffer_access_mask,
        buffer_stage_flags,
        object_name,
    );

    if let Some(staging_mem_buffer) = internal_state.staging_mem_buffer.as_ref() {
        let _ = vulkan_base
            .allocator
            .destroy_buffer(staging_mem_buffer.buffer, &staging_mem_buffer.allocation);
    }

    if let Some(command_pool) = internal_state.command_pool {
        unsafe {
            vulkan_base.device.destroy_command_pool(command_pool, None);
        }
    }

    res.map(|_| internal_state.gpu_mem_buffer.unwrap())
}

fn create_buffer_init_internal(
    internal_state: &mut InternalState,
    vulkan_base: &VulkanBase,
    init_data: &[u8],
    buffer_usage: vk::BufferUsageFlags,
    buffer_access_mask: vk::AccessFlags,
    buffer_stage_flags: vk::PipelineStageFlags,
    object_name: &str,
) -> Result<(), String> {
    let staging_mem_buffer =
        create_staging_buffer(vulkan_base, init_data.len() as vk::DeviceSize, object_name)?;

    unsafe {
        std::ptr::copy_nonoverlapping(
            init_data.as_ptr(),
            staging_mem_buffer.allocation_info.get_mapped_data(),
            init_data.len(),
        )
    };

    internal_state.staging_mem_buffer = Some(staging_mem_buffer);

    internal_state.gpu_mem_buffer = Some(create_gpu_buffer(
        vulkan_base,
        buffer_usage,
        init_data.len() as vk::DeviceSize,
        object_name,
    )?);

    let command_pool = create_command_pool(vulkan_base, object_name)?;

    internal_state.command_pool = Some(command_pool);

    let command_buffer = allocate_command_buffer(vulkan_base, command_pool, object_name)?;

    copy_buffer(
        vulkan_base,
        command_buffer,
        internal_state.staging_mem_buffer.as_ref().unwrap().buffer,
        internal_state.gpu_mem_buffer.as_ref().unwrap().buffer,
        buffer_access_mask,
        buffer_stage_flags,
        init_data.len() as vk::DeviceSize,
        object_name,
    )?;

    Ok(())
}

fn create_staging_buffer(
    vulkan_base: &VulkanBase,
    size: vk::DeviceSize,
    object_name: &str,
) -> Result<vulkan::MemBuffer, String> {
    let mem_buffer = vulkan::create_buffer(
        &vulkan_base.allocator,
        size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_VISIBLE,
        vk::MemoryPropertyFlags::HOST_COHERENT,
        vk_mem::AllocationCreateFlags::MAPPED,
    )
    .map_err(|_| format!("failed to create staging {}", object_name))?;

    vulkan::set_debug_utils_object_name(
        &vulkan_base.debug_utils_loader,
        vulkan_base.device.handle(),
        mem_buffer.buffer,
        &format!("staging {}", object_name),
    );

    vulkan::set_debug_utils_object_name(
        &vulkan_base.debug_utils_loader,
        vulkan_base.device.handle(),
        mem_buffer.allocation_info.get_device_memory(),
        &format!("staging {} device memory", object_name),
    );

    Ok(mem_buffer)
}

fn create_gpu_buffer(
    vulkan_base: &VulkanBase,
    buffer_usage: vk::BufferUsageFlags,
    size: vk::DeviceSize,
    object_name: &str,
) -> Result<vulkan::MemBuffer, String> {
    let mem_buffer = vulkan::create_buffer(
        &vulkan_base.allocator,
        size,
        buffer_usage | vk::BufferUsageFlags::TRANSFER_DST,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        vk::MemoryPropertyFlags::empty(),
        vk_mem::AllocationCreateFlags::NONE,
    )
    .map_err(|_| format!("failed to create {}", object_name))?;

    vulkan::set_debug_utils_object_name(
        &vulkan_base.debug_utils_loader,
        vulkan_base.device.handle(),
        mem_buffer.buffer,
        object_name,
    );

    vulkan::set_debug_utils_object_name(
        &vulkan_base.debug_utils_loader,
        vulkan_base.device.handle(),
        mem_buffer.allocation_info.get_device_memory(),
        &format!("{} device memory", object_name),
    );

    Ok(mem_buffer)
}

fn create_command_pool(
    vulkan_base: &VulkanBase,
    object_name: &str,
) -> Result<vk::CommandPool, String> {
    let create_info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::TRANSIENT)
        .queue_family_index(vulkan_base.queue_family)
        .build();

    let command_pool = unsafe {
        vulkan_base
            .device
            .create_command_pool(&create_info, None)
            .map_err(|_| format!("failed to create copy command pool for {}", object_name))?
    };

    vulkan::set_debug_utils_object_name(
        &vulkan_base.debug_utils_loader,
        vulkan_base.device.handle(),
        command_pool,
        &format!("copy command pool for {}", object_name),
    );

    Ok(command_pool)
}

fn allocate_command_buffer(
    vulkan_base: &VulkanBase,
    command_pool: vk::CommandPool,
    object_name: &str,
) -> Result<vk::CommandBuffer, String> {
    let allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(1)
        .build();

    let command_buffers = unsafe {
        vulkan_base
            .device
            .allocate_command_buffers(&allocate_info)
            .map_err(|_| format!("failed to allocate copy command buffer for {}", object_name))?
    };

    let command_buffer = command_buffers[0];

    vulkan::set_debug_utils_object_name(
        &vulkan_base.debug_utils_loader,
        vulkan_base.device.handle(),
        command_buffer,
        &format!("copy command buffer for {}", object_name),
    );

    Ok(command_buffer)
}

fn copy_buffer(
    vulkan_base: &VulkanBase,
    command_buffer: vk::CommandBuffer,
    src_buffer: vk::Buffer,
    dst_buffer: vk::Buffer,
    buffer_access_mask: vk::AccessFlags,
    buffer_stage_flags: vk::PipelineStageFlags,
    size: vk::DeviceSize,
    object_name: &str,
) -> Result<(), String> {
    let begin_info = vk::CommandBufferBeginInfo::builder()
        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
        .build();

    unsafe {
        vulkan_base
            .device
            .begin_command_buffer(command_buffer, &begin_info)
            .map_err(|_| format!("failed to begin copy command buffer for {}", object_name))?;

        let buffer_copy = vk::BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size,
        };

        vulkan_base
            .device
            .cmd_copy_buffer(command_buffer, src_buffer, dst_buffer, &[buffer_copy]);

        let after_copy_barrier = vk::BufferMemoryBarrier::builder()
            .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .dst_access_mask(buffer_access_mask)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .buffer(dst_buffer)
            .offset(0)
            .size(size)
            .build();

        vulkan_base.device.cmd_pipeline_barrier(
            command_buffer,
            vk::PipelineStageFlags::TRANSFER,
            buffer_stage_flags,
            vk::DependencyFlags::empty(),
            &[],
            &[after_copy_barrier],
            &[],
        );

        vulkan_base
            .device
            .end_command_buffer(command_buffer)
            .map_err(|_| format!("failed to end copy command buffer for {}", object_name))?;

        let cmd_buffers = [command_buffer];
        let submit_info = vk::SubmitInfo::builder()
            .command_buffers(&cmd_buffers)
            .build();

        vulkan_base
            .device
            .queue_submit(vulkan_base.queue, &[submit_info], vk::Fence::null())
            .map_err(|_| format!("failed to submit copy for {}", object_name))?;

        vulkan_base
            .device
            .queue_wait_idle(vulkan_base.queue)
            .map_err(|_| format!("failed to wait idle queue for {}", object_name))?;
    };

    Ok(())
}
