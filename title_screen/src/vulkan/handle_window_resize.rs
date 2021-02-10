use crate::vulkan;
use ash::version::DeviceV1_0;

pub fn handle_window_resize(
    data: &mut vulkan::TitleScreenVulkanData,
    vulkan_base_data: &mut vulkan_base::VulkanBaseData,
    window: &winit::window::Window,
    imgui_context: &mut imgui::Context,
) -> common::VulkanResult {
    unsafe {
        let _ = vulkan_base_data.get_device_ref().device_wait_idle();

        for &f in &data.framebuffers {
            vulkan_base_data
                .get_device_ref()
                .destroy_framebuffer(f, None);
        }
    }
    data.framebuffers.clear();

    vulkan_base::rebuild_swapchain_data(vulkan_base_data, window)?;
    vulkan::create_framebuffers(data, vulkan_base_data)?;

    // imgui_context.fonts().clear();
    // imgui_context.fonts().clear_fonts();
    // imgui_context.fonts().clear_tex_data();
    // imgui_context.fonts().clear_input_data();

    // let font_size = ((window.inner_size().width as f32) * crate::FONT_SIZE_RATIO).round();
    // imgui_context
    //     .fonts()
    //     .add_font(&[imgui::FontSource::TtfData {
    //         data: include_bytes!("../../resources/arial.ttf"),
    //         size_pixels: font_size,
    //         config: Some(imgui::FontConfig {
    //             name: Some(String::from("Arial")),
    //             ..imgui::FontConfig::default()
    //         }),
    //     }]);

    let mut imgui_fonts = imgui_context.fonts();
    let font_texture = imgui_fonts.build_rgba32_texture();

    let _ = vulkan_base_data
        .get_allocator_ref()
        .destroy_image(data.font_mem_image.image, &data.font_mem_image.allocation);

    unsafe {
        vulkan_base_data
            .get_device_ref()
            .destroy_image_view(data.font_mem_image.view, None);
        data.font_mem_image = Default::default();
    }

    vulkan::create_font_image(
        data,
        vulkan_base_data,
        &[font_texture.width, font_texture.height],
    )?;
    vulkan::create_font_image_view(data, vulkan_base_data)?;
    vulkan::copy_data_to_image(data, vulkan_base_data, font_texture.data)?;
    vulkan::update_descriptor_sets(data, vulkan_base_data);

    Ok(())
}
