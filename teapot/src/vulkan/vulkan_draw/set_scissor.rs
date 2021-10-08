use ash::vk;

use vulkan_base::VulkanBase;

pub fn set_scissor(vulkan_base: &VulkanBase, command_buffer: vk::CommandBuffer) {
    let scissor = vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent: vk::Extent2D {
            width: vulkan_base.surface_extent.width,
            height: vulkan_base.surface_extent.height,
        },
    };

    unsafe {
        vulkan_base
            .device
            .cmd_set_scissor(command_buffer, 0, &[scissor]);
    }
}
