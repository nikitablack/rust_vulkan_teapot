use crate::vulkan;
use ash::vk;
use vulkan_base::VulkanBase;

pub struct VulkanData {
    pub vertex_shader_module: vk::ShaderModule,
    pub tese_shader_module: vk::ShaderModule,
    pub tesc_shader_module: vk::ShaderModule,
    pub fragment_shader_module: vk::ShaderModule,
}

impl VulkanData {
    pub fn new(vulkan_base: &VulkanBase) -> Result<Self, String> {
        let vertex_sm = vulkan::create_shader_module(
            vulkan_base,
            std::path::Path::new("shaders/shader.vert.spv"),
            "vertex shader",
        )?;
        let vertex_sm_sg = scopeguard::guard(vertex_sm, |sm| {
            log::info!("something went wrong, destroying vertex shader module");
            unsafe {
                vulkan_base.device.destroy_shader_module(sm, None);
            }
        });

        let tese_sm = vulkan::create_shader_module(
            vulkan_base,
            std::path::Path::new("shaders/shader.tese.spv"),
            "tesselation evaluation shader",
        )?;
        let tese_sm_sg = scopeguard::guard(tese_sm, |sm| {
            log::info!("something went wrong, destroying tessellation evaluation shader module");
            unsafe {
                vulkan_base.device.destroy_shader_module(sm, None);
            }
        });

        let tesc_sm = vulkan::create_shader_module(
            vulkan_base,
            std::path::Path::new("shaders/shader.tesc.spv"),
            "tesselation control shader",
        )?;
        let tesc_sm_sg = scopeguard::guard(tesc_sm, |sm| {
            log::info!("something went wrong, destroying tessellation control shader module");
            unsafe {
                vulkan_base.device.destroy_shader_module(sm, None);
            }
        });

        let fragment_sm = vulkan::create_shader_module(
            vulkan_base,
            std::path::Path::new("shaders/shader.frag.spv"),
            "fragment shader",
        )?;

        let fragment_sm_sg = scopeguard::guard(fragment_sm, |sm| {
            log::info!("something went wrong, destroying fragment shader module");
            unsafe {
                vulkan_base.device.destroy_shader_module(sm, None);
            }
        });

        Ok(VulkanData {
            vertex_shader_module: scopeguard::ScopeGuard::into_inner(vertex_sm_sg),
            tese_shader_module: scopeguard::ScopeGuard::into_inner(tese_sm_sg),
            tesc_shader_module: scopeguard::ScopeGuard::into_inner(tesc_sm_sg),
            fragment_shader_module: scopeguard::ScopeGuard::into_inner(fragment_sm_sg),
        })
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
