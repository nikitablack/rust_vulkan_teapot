pub struct MemImage {
    pub image: ash::vk::Image,
    pub view: ash::vk::ImageView,
    pub extent: ash::vk::Extent3D,
    pub allocation: gpu_allocator::vulkan::Allocation,
}

impl Default for MemImage {
    fn default() -> Self {
        Self {
            image: ash::vk::Image::null(),
            view: ash::vk::ImageView::null(),
            extent: ash::vk::Extent3D::default(),
            allocation: gpu_allocator::vulkan::Allocation::default(),
        }
    }
}
