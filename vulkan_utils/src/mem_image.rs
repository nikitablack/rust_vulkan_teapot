pub struct MemImage {
    pub image: ash::vk::Image,
    pub view: ash::vk::ImageView,
    pub extent: ash::vk::Extent3D,
    pub allocation: gpu_allocator::vulkan::Allocation,
}
