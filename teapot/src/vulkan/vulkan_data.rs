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
        let vertex_sm_sg = {
            let vertex_sm = vulkan_utils::create_shader_module(
                &vulkan_base.device,
                std::path::Path::new("shaders/shader.vert.spv"),
                &vulkan_base.debug_utils_loader,
                "vertex shader",
            )?;

            scopeguard::guard(vertex_sm, |sm| {
                log::warn!("vertex shader scopeguard");
                unsafe {
                    vulkan_base.device.destroy_shader_module(sm, None);
                }
            })
        };

        let tese_sm_sg = {
            let tese_sm = vulkan_utils::create_shader_module(
                &vulkan_base.device,
                std::path::Path::new("shaders/shader.tese.spv"),
                &vulkan_base.debug_utils_loader,
                "tessellation evaluation shader",
            )?;

            scopeguard::guard(tese_sm, |sm| {
                log::warn!("tessellation evaluation shader scopeguard");
                unsafe {
                    vulkan_base.device.destroy_shader_module(sm, None);
                }
            })
        };

        let tesc_sm_sg = {
            let tesc_sm = vulkan_utils::create_shader_module(
                &vulkan_base.device,
                std::path::Path::new("shaders/shader.tesc.spv"),
                &vulkan_base.debug_utils_loader,
                "tessellation control shader",
            )?;

            scopeguard::guard(tesc_sm, |sm| {
                log::warn!("tessellation control shader scopeguard");
                unsafe {
                    vulkan_base.device.destroy_shader_module(sm, None);
                }
            })
        };

        let fragment_sm_sg = {
            let fragment_sm = vulkan_utils::create_shader_module(
                &vulkan_base.device,
                std::path::Path::new("shaders/shader.frag.spv"),
                &vulkan_base.debug_utils_loader,
                "fragment shader",
            )?;

            scopeguard::guard(fragment_sm, |sm| {
                log::warn!("fragment shader scopeguard");
                unsafe {
                    vulkan_base.device.destroy_shader_module(sm, None);
                }
            })
        };

        Ok(VulkanData {
            vertex_shader_module: scopeguard::ScopeGuard::into_inner(vertex_sm_sg),
            tese_shader_module: scopeguard::ScopeGuard::into_inner(tese_sm_sg),
            tesc_shader_module: scopeguard::ScopeGuard::into_inner(tesc_sm_sg),
            fragment_shader_module: scopeguard::ScopeGuard::into_inner(fragment_sm_sg),
        })
    }

    pub fn clean(self, vulkan_base: &VulkanBase) {
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
