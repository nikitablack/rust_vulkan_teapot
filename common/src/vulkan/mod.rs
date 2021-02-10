mod constants;
mod create_buffer;
mod create_shader_module;
mod set_debug_utils_object_name;

pub use constants::*;
pub use create_buffer::*;
pub use create_shader_module::*;
pub use set_debug_utils_object_name::*;

use ash::vk;

pub struct MemImage {
    pub image: vk::Image,
    pub view: vk::ImageView,
    pub allocation: vk_mem::Allocation,
    pub allocation_info: Option<vk_mem::AllocationInfo>,
    pub extent: vk::Extent3D,
}

impl Default for MemImage {
    fn default() -> Self {
        Self {
            image: vk::Image::null(),
            view: vk::ImageView::null(),
            allocation: vk_mem::Allocation::null(),
            allocation_info: None,
            extent: vk::Extent3D::default(),
        }
    }
}

impl MemImage {
    pub fn get_allocation_info_ref(&self) -> &vk_mem::AllocationInfo {
        self.allocation_info
            .as_ref()
            .expect("allocation info shouldn't be empty")
    }
}

pub struct MemBuffer {
    pub buffer: vk::Buffer,
    pub size: vk::DeviceSize,
    pub allocation: vk_mem::Allocation,
    pub allocation_info: Option<vk_mem::AllocationInfo>,
}

impl Default for MemBuffer {
    fn default() -> Self {
        Self {
            buffer: vk::Buffer::null(),
            size: 0,
            allocation: vk_mem::Allocation::null(),
            allocation_info: None,
        }
    }
}

impl MemBuffer {
    pub fn get_allocation_info_ref(&self) -> &vk_mem::AllocationInfo {
        self.allocation_info
            .as_ref()
            .expect("allocation info shouldn't be empty")
    }
}

pub type VulkanResult = Result<(), String>;
