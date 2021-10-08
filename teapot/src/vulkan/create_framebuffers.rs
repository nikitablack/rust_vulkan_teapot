use ash::vk;

pub fn create_framebuffers(
    device: &ash::Device,
    swapchain_image_views: &Vec<vk::ImageView>,
    render_pass: vk::RenderPass,
    framebuffer_extent: vk::Extent2D,
    depth_buffer_view: vk::ImageView,
    debug_utils_loader: &ash::extensions::ext::DebugUtils,
) -> Result<Vec<vk::Framebuffer>, String> {
    let mut framebuffers = Vec::with_capacity(swapchain_image_views.len());

    for (i, &view) in swapchain_image_views.iter().enumerate() {
        let attachments = [view, depth_buffer_view];

        let create_info = vk::FramebufferCreateInfo::builder()
            .render_pass(render_pass)
            .attachments(&attachments)
            .width(framebuffer_extent.width)
            .height(framebuffer_extent.height)
            .layers(1)
            .build();

        let framebuffer = unsafe {
            device.create_framebuffer(&create_info, None).map_err(|_| {
                for &fb in &framebuffers {
                    device.destroy_framebuffer(fb, None);
                }
                format!("failed to create framebuffer {}", i)
            })?
        };

        framebuffers.push(framebuffer);

        vulkan_utils::set_debug_utils_object_name(
            debug_utils_loader,
            device.handle(),
            framebuffer,
            &format!("framebuffer {}", i),
        );
    }

    Ok(framebuffers)
}
