pub struct MemBuffer {
    pub buffer: ash::vk::Buffer,
    pub size: ash::vk::DeviceSize,
    pub allocation: gpu_allocator::vulkan::Allocation,
}
