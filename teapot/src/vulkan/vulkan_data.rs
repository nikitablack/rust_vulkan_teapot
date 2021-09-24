use crate::vulkan;
use ash::vk;
use vulkan_base::VulkanBase;

#[derive(Default)]
pub struct VulkanData {
    pub vertex_shader_module: vk::ShaderModule,
    pub tese_shader_module: vk::ShaderModule,
    pub tesc_shader_module: vk::ShaderModule,
    pub fragment_shader_module: vk::ShaderModule,
}

impl VulkanData {
    pub fn new(vulkan_base: &VulkanBase) -> Result<Self, String> {
        let mut vulkan_data = VulkanData::default();

        if let Err(msg) = new_internal(&mut vulkan_data, vulkan_base) {
            vulkan_data.clean(vulkan_base);
            return Err(msg);
        }

        Ok(vulkan_data)
    }

    pub fn clean(&self, vulkan_base: &VulkanBase) {
        log::info!("cleaning vulkan data");

        unsafe {
            vulkan_base
                .device
                .destroy_shader_module(self.vertex_shader_module, None);
            vulkan_base
                .device
                .destroy_shader_module(self.tese_shader_module, None);
            vulkan_base
                .device
                .destroy_shader_module(self.tesc_shader_module, None);
            vulkan_base
                .device
                .destroy_shader_module(self.fragment_shader_module, None);
        }
    }
}

fn new_internal(vulkan_data: &mut VulkanData, vulkan_base: &VulkanBase) -> Result<(), String> {
    vulkan_data.vertex_shader_module = vulkan::create_shader_module(
        vulkan_base,
        std::path::Path::new("shaders/shader.vert.spv"),
        "vertex shader",
    )?;

    vulkan_data.tese_shader_module = vulkan::create_shader_module(
        vulkan_base,
        std::path::Path::new("shaders/shader.tese.spv"),
        "tesselation evaluation shader",
    )?;

    vulkan_data.tesc_shader_module = vulkan::create_shader_module(
        vulkan_base,
        std::path::Path::new("shaders/shader.tesc.spv"),
        "tesselation control shader",
    )?;

    vulkan_data.fragment_shader_module = vulkan::create_shader_module(
        vulkan_base,
        std::path::Path::new("shaders/shader.frag.spv"),
        "fragment shader",
    )?;

    Ok(())
}
