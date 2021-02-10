use vulkan::TitleScreenVulkanData;

mod ui;
pub mod vulkan;

pub use ui::*;

#[macro_use]
extern crate static_assertions;

fn init_internal(
    data: &mut TitleScreenVulkanData,
    vulkan_base_data: &mut vulkan_base::VulkanBaseData,
    font_texture: &imgui::FontAtlasTexture,
) -> common::VulkanResult {
    vulkan::create_render_pass(data, vulkan_base_data)?;
    vulkan::create_framebuffers(data, vulkan_base_data)?;
    vulkan::create_vertex_shader_module(data, vulkan_base_data)?;
    vulkan::create_fragment_shader_module(data, vulkan_base_data)?;
    vulkan::create_command_pool(data, vulkan_base_data)?;
    vulkan::create_vertex_buffers(data, vulkan_base_data)?;
    vulkan::create_index_buffers(data, vulkan_base_data)?;
    vulkan::create_descriptor_pool(data, vulkan_base_data)?;
    vulkan::create_sampler(data, vulkan_base_data)?;
    vulkan::create_descriptor_set_layout(data, vulkan_base_data)?;
    vulkan::create_pipeline_layout(data, vulkan_base_data)?;
    vulkan::create_fences(data, vulkan_base_data)?;
    vulkan::create_semaphores(data, vulkan_base_data)?;
    vulkan::create_pipeline(data, vulkan_base_data)?;
    vulkan::allocate_descriptor_sets(data, vulkan_base_data)?;
    vulkan::create_font_image(
        data,
        vulkan_base_data,
        &[font_texture.width, font_texture.height],
    )?;
    vulkan::create_font_image_view(data, vulkan_base_data)?;
    vulkan::copy_data_to_image(data, vulkan_base_data, font_texture.data)?;
    vulkan::update_descriptor_sets(data, vulkan_base_data);

    data.command_buffers =
        vec![ash::vk::CommandBuffer::null(); common::NUM_RESOURCES_IN_FLIGHT as usize];

    Ok(())
}

pub fn init_vulkan(
    vulkan_base_data: &mut vulkan_base::VulkanBaseData,
    font_texture: &imgui::FontAtlasTexture,
) -> Result<vulkan::TitleScreenVulkanData, String> {
    let mut data = vulkan::TitleScreenVulkanData::default();

    if let Err(msg) = init_internal(&mut data, vulkan_base_data, font_texture) {
        vulkan::clear_title_screen_data(&mut data, vulkan_base_data);
        return Err(msg);
    }

    Ok(data)
}

pub fn draw(
    data: &mut vulkan::TitleScreenVulkanData,
    vulkan_base_data: &vulkan_base::VulkanBaseData,
    ui_draw_data: &imgui::DrawData,
) -> common::VulkanResult {
    vulkan::wait_resource_available(data, &vulkan_base_data)?;
    vulkan::copy_ui_data(data, &vulkan_base_data, ui_draw_data)?;
    vulkan::draw(data, &vulkan_base_data, ui_draw_data)?;

    data.resource_index += 1;
    data.resource_index %= common::NUM_RESOURCES_IN_FLIGHT;

    Ok(())
}
