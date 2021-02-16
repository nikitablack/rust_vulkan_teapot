use crate::vulkan;
use ash::{version::DeviceV1_0, vk};
use vulkan_base::VulkanBase;

pub struct VulkanData {
    pub vertex_shader_module: vk::ShaderModule,
    pub tese_shader_module: vk::ShaderModule,
    pub tesc_shader_module: vk::ShaderModule,
    pub fragment_shader_module: vk::ShaderModule,
}

impl VulkanData {
    pub fn new(vulkan_base: &VulkanBase) -> Result<Self, String> {
        let vertex_shader_module = vulkan::create_shader_module(
            &vulkan_base.device,
            std::path::Path::new("shaders/shader.vert.spv"),
        )?;

        vulkan::set_debug_utils_object_name(
            &vulkan_base.debug_utils_loader,
            vulkan_base.device.handle(),
            vertex_shader_module,
            "vertex shader",
        );

        let tese_shader_module = vulkan::create_shader_module(
            &vulkan_base.device,
            std::path::Path::new("shaders/shader.tese.spv"),
        )?;

        vulkan::set_debug_utils_object_name(
            &vulkan_base.debug_utils_loader,
            vulkan_base.device.handle(),
            tese_shader_module,
            "tesselation evaluation shader",
        );

        let tesc_shader_module = vulkan::create_shader_module(
            &vulkan_base.device,
            std::path::Path::new("shaders/shader.tesc.spv"),
        )?;

        vulkan::set_debug_utils_object_name(
            &vulkan_base.debug_utils_loader,
            vulkan_base.device.handle(),
            tesc_shader_module,
            "tesselation control shader",
        );

        let fragment_shader_module = vulkan::create_shader_module(
            &vulkan_base.device,
            std::path::Path::new("shaders/shader.frag.spv"),
        )?;

        vulkan::set_debug_utils_object_name(
            &vulkan_base.debug_utils_loader,
            vulkan_base.device.handle(),
            fragment_shader_module,
            "fragment shader",
        );

        let vulkan_data = VulkanData {
            vertex_shader_module,
            tese_shader_module,
            tesc_shader_module,
            fragment_shader_module,
        };

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
