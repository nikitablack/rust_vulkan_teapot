use crate::vulkan;

pub fn create_vertex_shader_module(
    data: &mut vulkan::TitleScreenVulkanData,
    vulkan_base_data: &vulkan_base::VulkanBaseData,
) -> common::VulkanResult {
    let device = vulkan_base_data.get_device_ref();

    data.vertex_shader_module = match common::create_shader_module(
        device,
        std::path::Path::new("title_screen/resources/vertex_shader.vert.spv"),
    ) {
        Ok(module) => module,
        Err(msg) => return Err(msg),
    };

    if let Some(debug_utils) = vulkan_base_data.debug_utils_loader.as_ref() {
        common::set_debug_utils_object_name(
            debug_utils,
            device.handle(),
            data.vertex_shader_module,
            String::from("title_screen_vertex shader module"),
        );
    }

    Ok(())
}
