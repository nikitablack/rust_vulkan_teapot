use ash::vk;

use vulkan_base::VulkanBase;

pub fn set_viewport(vulkan_base: &VulkanBase, command_buffer: vk::CommandBuffer) {
    let viewport = vk::Viewport {
        x: 0.0,
        y: 0.0,
        width: vulkan_base.surface_extent.width as f32,
        height: vulkan_base.surface_extent.height as f32,
        min_depth: 0.0f32,
        max_depth: 1.0f32,
    };

    unsafe {
        vulkan_base
            .device
            .cmd_set_viewport(command_buffer, 0, &[viewport]);
    }
}
