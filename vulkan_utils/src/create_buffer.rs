use std::cell::RefCell;

use crate::MemBuffer;
use ash::vk;

pub fn create_buffer(
    device: &ash::Device,
    allocator: &mut gpu_allocator::vulkan::Allocator,
    debug_utils_loader: &ash::extensions::ext::DebugUtils,
    size: vk::DeviceSize,
    buffer_usage: vk::BufferUsageFlags,
    memory_location: gpu_allocator::MemoryLocation,
    object_name: &str,
) -> Result<MemBuffer, String> {
    // buffer
    log::info!("{}: creating", object_name);

    let buffer_create_info = vk::BufferCreateInfo::builder()
        .size(size)
        .usage(buffer_usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let buffer_sg = {
        let buffer = unsafe {
            device
                .create_buffer(&buffer_create_info, None)
                .map_err(|_| format!("{}: failed to create", object_name))?
        };

        scopeguard::guard(buffer, |buffer| {
            log::warn!("{} scopeguard", object_name);
            unsafe {
                device.destroy_buffer(buffer, None);
            }
        })
    };

    log::info!("{}: created", object_name);

    // allocation
    log::info!("{}: allocating memory", object_name);

    let memory_requirements = unsafe { device.get_buffer_memory_requirements(*buffer_sg) };

    let allocation_create_desc = gpu_allocator::vulkan::AllocationCreateDesc {
        name: object_name,
        requirements: memory_requirements,
        location: memory_location,
        linear: true,
    };

    let allocation_sg = {
        let allocation = allocator
            .allocate(&allocation_create_desc)
            .map_err(|_| format!("{}: failed to allocate memory", object_name))?;

        scopeguard::guard(allocation, |allocation| {
            log::warn!("{} allocation scopeguard", object_name);
            let _ = allocator.free(allocation);
        })
    };

    log::info!("{}: memory allocated", object_name);

    // binding
    log::info!("{}: binding memory", object_name);

    unsafe {
        device
            .bind_buffer_memory(*buffer_sg, allocation_sg.memory(), allocation_sg.offset())
            .map_err(|_| format!("{}: failed to bind memory", object_name))?
    };

    log::info!("{}: memory bound", object_name);

    crate::set_debug_utils_object_name(
        debug_utils_loader,
        device.handle(),
        *buffer_sg,
        object_name,
    );

    crate::set_debug_utils_object_name(
        &debug_utils_loader,
        device.handle(),
        unsafe { allocation_sg.memory() },
        &format!("{} memory", object_name),
    );

    Ok(MemBuffer {
        buffer: scopeguard::ScopeGuard::into_inner(buffer_sg),
        allocation: scopeguard::ScopeGuard::into_inner(allocation_sg),
    })
}

pub fn create_gpu_buffer_init(
    device: &ash::Device,
    allocator: &mut gpu_allocator::vulkan::Allocator,
    debug_utils_loader: &ash::extensions::ext::DebugUtils,
    queue_family: u32,
    queue: vk::Queue,
    init_data: &[u8],
    buffer_usage: vk::BufferUsageFlags,
    buffer_access_mask: vk::AccessFlags,
    buffer_stage_flags: vk::PipelineStageFlags,
    object_name: &str,
) -> Result<MemBuffer, String> {
    let allocator_rc = RefCell::new(allocator);

    // staging buffer
    log::info!("{}: creating with data", object_name);

    let mut staging_mem_buffer_sg = {
        let staging_mem_buffer = create_buffer(
            device,
            *allocator_rc.borrow_mut(),
            debug_utils_loader,
            init_data.len() as vk::DeviceSize,
            vk::BufferUsageFlags::TRANSFER_SRC,
            gpu_allocator::MemoryLocation::CpuToGpu,
            &format!("{} staging", object_name),
        )?;

        scopeguard::guard(staging_mem_buffer, |mem_buffer| {
            log::warn!("{} staging scopeguard", object_name);
            unsafe {
                device.destroy_buffer(mem_buffer.buffer, None);
            }
            let _ = allocator_rc.borrow_mut().free(mem_buffer.allocation);
        })
    };

    // copy data to staging memory
    log::info!("{} staging: copying data to mapped memory", object_name);

    staging_mem_buffer_sg.allocation.mapped_slice_mut().unwrap()[..init_data.len()]
        .copy_from_slice(init_data);

    // gpu buffer
    let gpu_mem_buffer_sg = {
        let gpu_mem_buffer = create_buffer(
            device,
            *allocator_rc.borrow_mut(),
            debug_utils_loader,
            init_data.len() as vk::DeviceSize,
            buffer_usage | vk::BufferUsageFlags::TRANSFER_DST,
            gpu_allocator::MemoryLocation::GpuOnly,
            object_name,
        )?;

        scopeguard::guard(gpu_mem_buffer, |mem_buffer| {
            log::warn!("{} scopeguard", object_name);
            unsafe {
                device.destroy_buffer(mem_buffer.buffer, None);
            }
            let _ = allocator_rc.borrow_mut().free(mem_buffer.allocation);
        })
    };

    // command pool
    let command_pool_sg = {
        let command_pool = create_command_pool(device, queue_family, object_name)?;
        scopeguard::guard(command_pool, |command_pool| {
            log::warn!("{} command pool scopeguard", object_name);
            unsafe {
                device.destroy_command_pool(command_pool, None);
            }
        })
    };

    // command buffer
    let command_buffer = allocate_command_buffer(device, *command_pool_sg, object_name)?;
    // no need to free explicitly, it will be freed implicitly on command pool destruction

    // copy staging memory to gpu memory
    copy_buffer(
        device,
        queue,
        command_buffer,
        staging_mem_buffer_sg.buffer,
        gpu_mem_buffer_sg.buffer,
        buffer_access_mask,
        buffer_stage_flags,
        init_data.len() as vk::DeviceSize,
        object_name,
    )?;

    // clear temporary objects
    log::info!("{}: destroying temporary objects", object_name);

    let staging_mem_buffer = scopeguard::ScopeGuard::into_inner(staging_mem_buffer_sg);

    unsafe {
        device.destroy_buffer(staging_mem_buffer.buffer, None);
        let _ = allocator_rc
            .borrow_mut()
            .free(staging_mem_buffer.allocation);
        device.destroy_command_pool(scopeguard::ScopeGuard::into_inner(command_pool_sg), None);
    }

    let gpu_mem_buffer = scopeguard::ScopeGuard::into_inner(gpu_mem_buffer_sg);

    Ok(gpu_mem_buffer)
}

fn create_command_pool(
    device: &ash::Device,
    queue_family: u32,
    object_name: &str,
) -> Result<vk::CommandPool, String> {
    log::info!("{}: creating command pool", object_name);

    let create_info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::TRANSIENT)
        .queue_family_index(queue_family)
        .build();

    let command_pool = unsafe {
        device
            .create_command_pool(&create_info, None)
            .map_err(|_| format!("{}: failed to create command pool", object_name))?
    };

    log::info!("{}: command pool created", object_name);

    Ok(command_pool)
}

fn allocate_command_buffer(
    device: &ash::Device,
    command_pool: vk::CommandPool,
    object_name: &str,
) -> Result<vk::CommandBuffer, String> {
    log::info!("{}: allocating command buffer", object_name);

    let allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(1)
        .build();

    let command_buffers = unsafe {
        device
            .allocate_command_buffers(&allocate_info)
            .map_err(|_| format!("{}: failed to allocate copy command buffer", object_name))?
    };

    log::info!("{}: command buffer allocated", object_name);

    Ok(command_buffers[0])
}

fn copy_buffer(
    device: &ash::Device,
    queue: vk::Queue,
    command_buffer: vk::CommandBuffer,
    src_buffer: vk::Buffer,
    dst_buffer: vk::Buffer,
    buffer_access_mask: vk::AccessFlags,
    buffer_stage_flags: vk::PipelineStageFlags,
    size: vk::DeviceSize,
    object_name: &str,
) -> Result<(), String> {
    log::info!("{}: copying buffer to buffer", object_name);

    let begin_info = vk::CommandBufferBeginInfo::builder()
        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
        .build();

    unsafe {
        device
            .begin_command_buffer(command_buffer, &begin_info)
            .map_err(|_| format!("{}: failed to begin copy command buffer", object_name))?;

        let buffer_copy = vk::BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size,
        };

        device.cmd_copy_buffer(command_buffer, src_buffer, dst_buffer, &[buffer_copy]);

        let after_copy_barrier = vk::BufferMemoryBarrier::builder()
            .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .dst_access_mask(buffer_access_mask)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .buffer(dst_buffer)
            .offset(0)
            .size(size)
            .build();

        device.cmd_pipeline_barrier(
            command_buffer,
            vk::PipelineStageFlags::TRANSFER,
            buffer_stage_flags,
            vk::DependencyFlags::empty(),
            &[],
            &[after_copy_barrier],
            &[],
        );

        device
            .end_command_buffer(command_buffer)
            .map_err(|_| format!("{}: failed to end copy command buffer", object_name))?;

        let cmd_buffers = [command_buffer];
        let submit_info = vk::SubmitInfo::builder()
            .command_buffers(&cmd_buffers)
            .build();

        device
            .queue_submit(queue, &[submit_info], vk::Fence::null())
            .map_err(|_| format!("{}: failed to submit copy", object_name))?;

        device
            .queue_wait_idle(queue)
            .map_err(|_| format!("{}: failed to wait idle queue", object_name))?;

        log::info!("{}: buffer to buffer copied", object_name);
    };

    Ok(())
}
