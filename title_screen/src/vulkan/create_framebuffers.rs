use crate::vulkan;
use ash::version::DeviceV1_0;
use ash::vk;

pub fn create_framebuffers(
    data: &mut vulkan::TitleScreenVulkanData,
    vulkan_base_data: &vulkan_base::VulkanBaseData,
) -> common::VulkanResult {
    debug_assert!(data.framebuffers.is_empty());

    let device = vulkan_base_data.get_device_ref();

    data.framebuffers
        .reserve(vulkan_base_data.swapchain_image_views.len());

    for (i, &view) in vulkan_base_data.swapchain_image_views.iter().enumerate() {
        let attachments = [view];

        let create_info = vk::FramebufferCreateInfo::builder()
            .render_pass(data.render_pass)
            .attachments(&attachments)
            .width(vulkan_base_data.surface_extent.width)
            .height(vulkan_base_data.surface_extent.height)
            .layers(1);

        let framebuffer = match unsafe { device.create_framebuffer(&create_info, None) } {
            Ok(fb) => fb,
            Err(_) => return Err(format!("failed to create framebuffer {}", i)),
        };

        data.framebuffers.push(framebuffer);

        if let Some(debug_utils) = vulkan_base_data.debug_utils_loader.as_ref() {
            common::set_debug_utils_object_name(
                debug_utils,
                device.handle(),
                framebuffer,
                format!("title screen framebuffer {}", i),
            );
        }
    }

    Ok(())
}
