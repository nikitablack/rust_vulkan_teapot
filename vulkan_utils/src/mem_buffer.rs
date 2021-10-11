pub struct MemBuffer {
    pub buffer: ash::vk::Buffer,
    pub allocation: gpu_allocator::vulkan::Allocation,
}
