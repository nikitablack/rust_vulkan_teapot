use crate::vulkan;
use ash::{version::DeviceV1_0, vk};
use vulkan_base::VulkanBase;

pub struct VulkanData {
    pub vertex_shader_module: vk::ShaderModule,
    pub tese_shader_module: vk::ShaderModule,
    pub tesc_shader_module: vk::ShaderModule,
    pub fragment_shader_module: vk::ShaderModule,
}

#[derive(Default)]
struct InternalState {
    vertex_shader_module: vk::ShaderModule,
    tese_shader_module: vk::ShaderModule,
    tesc_shader_module: vk::ShaderModule,
    fragment_shader_module: vk::ShaderModule,
}

impl VulkanData {
    pub fn new(vulkan_base: &VulkanBase) -> Result<Self, String> {
        let mut internal_state = InternalState::default();

        if let Err(msg) = new_internal(&mut internal_state, vulkan_base) {
            clean_internal(&internal_state, vulkan_base);
            return Err(msg);
        }

        let vulkan_data = VulkanData {
            vertex_shader_module: internal_state.vertex_shader_module,
            tese_shader_module: internal_state.tese_shader_module,
            tesc_shader_module: internal_state.tesc_shader_module,
            fragment_shader_module: internal_state.fragment_shader_module,
        };

        Ok(vulkan_data)
    }

    pub fn clean(&self, vulkan_base: &VulkanBase) {
        log::info!("cleaning vulkan data");

        let internal_state = InternalState {
            vertex_shader_module: self.vertex_shader_module,
            tese_shader_module: self.tese_shader_module,
            tesc_shader_module: self.tesc_shader_module,
            fragment_shader_module: self.fragment_shader_module,
        };

        clean_internal(&internal_state, vulkan_base);
    }
}

fn new_internal(
    internal_state: &mut InternalState,
    vulkan_base: &VulkanBase,
) -> Result<(), String> {
    internal_state.vertex_shader_module = vulkan::create_shader_module(
        &vulkan_base.device,
        std::path::Path::new("shaders/shader.vert.spv"),
    )?;

    vulkan::set_debug_utils_object_name(
        &vulkan_base.debug_utils_loader,
        vulkan_base.device.handle(),
        internal_state.vertex_shader_module,
        "vertex shader",
    );

    internal_state.tese_shader_module = vulkan::create_shader_module(
        &vulkan_base.device,
        std::path::Path::new("shaders/shader.tese.spv"),
    )?;

    vulkan::set_debug_utils_object_name(
        &vulkan_base.debug_utils_loader,
        vulkan_base.device.handle(),
        internal_state.tese_shader_module,
        "tesselation evaluation shader",
    );

    internal_state.tesc_shader_module = vulkan::create_shader_module(
        &vulkan_base.device,
        std::path::Path::new("shaders/shader.tesc.spv"),
    )?;

    vulkan::set_debug_utils_object_name(
        &vulkan_base.debug_utils_loader,
        vulkan_base.device.handle(),
        internal_state.tesc_shader_module,
        "tesselation control shader",
    );

    internal_state.fragment_shader_module = vulkan::create_shader_module(
        &vulkan_base.device,
        std::path::Path::new("shaders/shader.frag.spv"),
    )?;

    vulkan::set_debug_utils_object_name(
        &vulkan_base.debug_utils_loader,
        vulkan_base.device.handle(),
        internal_state.fragment_shader_module,
        "fragment shader",
    );

    Ok(())
}

fn clean_internal(internal_state: &InternalState, vulkan_base: &VulkanBase) {
    unsafe {
        vulkan_base
            .device
            .destroy_shader_module(internal_state.vertex_shader_module, None);
        vulkan_base
            .device
            .destroy_shader_module(internal_state.tese_shader_module, None);
        vulkan_base
            .device
            .destroy_shader_module(internal_state.tesc_shader_module, None);
        vulkan_base
            .device
            .destroy_shader_module(internal_state.fragment_shader_module, None);
    }
}
