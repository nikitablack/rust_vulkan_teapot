use ash::vk;

use crate::vulkan;

pub fn create_buffer(
    allocator: &vk_mem::Allocator,
    size: vk::DeviceSize,
    buffer_usage: vk::BufferUsageFlags,
    memory_required_flags: vk::MemoryPropertyFlags,
    memory_preferred_flags: vk::MemoryPropertyFlags,
    allocation_flags: vk_mem::AllocationCreateFlags,
) -> Result<vulkan::MemBuffer, vk_mem::Error> {
    let buffer_create_info = vk::BufferCreateInfo::builder()
        .size(size)
        .usage(buffer_usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .build();

    let allocation_create_info = vk_mem::AllocationCreateInfo {
        usage: vk_mem::MemoryUsage::Unknown,
        flags: allocation_flags,
        required_flags: memory_required_flags,
        preferred_flags: memory_preferred_flags,
        ..Default::default()
    };

    let (buffer, allocation, allocation_info) =
        allocator.create_buffer(&buffer_create_info, &allocation_create_info)?;

    Ok(vulkan::MemBuffer {
        buffer,
        size,
        allocation,
        allocation_info,
    })
}
