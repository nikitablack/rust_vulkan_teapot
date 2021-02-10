use ash::vk;

use crate::vulkan;

pub fn create_buffer(
    allocator: &vk_mem::Allocator,
    size: vk::DeviceSize,
    buffer_usage: vk::BufferUsageFlags,
    memory_usage: vk_mem::MemoryUsage,
    memory_flags: vk_mem::AllocationCreateFlags,
) -> Result<vulkan::MemBuffer, vk_mem::Error> {
    let buffer_create_info = vk::BufferCreateInfo::builder()
        .size(size)
        .usage(buffer_usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let allocation_create_info = vk_mem::AllocationCreateInfo {
        usage: memory_usage,
        flags: memory_flags,
        ..Default::default()
    };

    let (buffer, allocation, info) =
        allocator.create_buffer(&buffer_create_info, &allocation_create_info)?;

    Ok(vulkan::MemBuffer {
        buffer,
        size,
        allocation,
        allocation_info: Some(info),
    })
}
