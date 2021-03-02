use crate::vulkan;
use ash::version::DeviceV1_0;
use ash::vk;

pub fn create_framebuffers(
    vulkan_base: &vulkan_base::VulkanBase,
    render_pass: vk::RenderPass,
    framebuffer_extent: vk::Extent2D,
) -> Result<Vec<vk::Framebuffer>, String> {
    let mut framebuffers = Vec::with_capacity(vulkan_base.swapchain_image_views.len());

    for (i, &view) in vulkan_base.swapchain_image_views.iter().enumerate() {
        let attachments = [view];

        let create_info = vk::FramebufferCreateInfo::builder()
            .render_pass(render_pass)
            .attachments(&attachments)
            .width(framebuffer_extent.width)
            .height(framebuffer_extent.height)
            .layers(1)
            .build();

        let framebuffer = unsafe {
            vulkan_base
                .device
                .create_framebuffer(&create_info, None)
                .map_err(|_| {
                    for &fb in &framebuffers {
                        vulkan_base.device.destroy_framebuffer(fb, None);
                    }
                    format!("failed to create framebuffer {}", i)
                })?
        };

        framebuffers.push(framebuffer);

        vulkan::set_debug_utils_object_name(
            &vulkan_base.debug_utils_loader,
            vulkan_base.device.handle(),
            framebuffer,
            &format!("framebuffer {}", i),
        );
    }

    Ok(framebuffers)
}
