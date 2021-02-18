use crate::{teapot_data, vulkan};
use ash::{version::DeviceV1_0, vk};
use vulkan::create_buffer_init;
use vulkan_base::VulkanBase;

#[derive(Clone)]
pub struct MemBuffer {
    pub buffer: vk::Buffer,
    pub size: vk::DeviceSize,
    pub allocation: vk_mem::Allocation,
    pub allocation_info: vk_mem::AllocationInfo,
}

pub struct VulkanData {
    pub vertex_shader_module: vk::ShaderModule,
    pub tese_shader_module: vk::ShaderModule,
    pub tesc_shader_module: vk::ShaderModule,
    pub fragment_shader_module: vk::ShaderModule,
    pub control_points_mem_buffer: MemBuffer,
}

#[derive(Default)]
struct InternalState {
    vertex_shader_module: vk::ShaderModule,
    tese_shader_module: vk::ShaderModule,
    tesc_shader_module: vk::ShaderModule,
    fragment_shader_module: vk::ShaderModule,
    control_points_mem_buffer: Option<MemBuffer>,
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
            control_points_mem_buffer: internal_state.control_points_mem_buffer.unwrap(),
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
            control_points_mem_buffer: Some(self.control_points_mem_buffer.clone()),
        };

        clean_internal(&internal_state, vulkan_base);
    }
}

fn new_internal(
    internal_state: &mut InternalState,
    vulkan_base: &VulkanBase,
) -> Result<(), String> {
    internal_state.vertex_shader_module = create_shader_module(
        vulkan_base,
        std::path::Path::new("shaders/shader.vert.spv"),
        "vertex shader",
    )?;

    internal_state.tese_shader_module = create_shader_module(
        vulkan_base,
        std::path::Path::new("shaders/shader.tese.spv"),
        "tesselation evaluation shader",
    )?;

    internal_state.tesc_shader_module = create_shader_module(
        vulkan_base,
        std::path::Path::new("shaders/shader.tesc.spv"),
        "tesselation control shader",
    )?;

    internal_state.fragment_shader_module = create_shader_module(
        vulkan_base,
        std::path::Path::new("shaders/shader.frag.spv"),
        "fragment shader",
    )?;

    let teapot_data = teapot_data::TeapotData::new();

    internal_state.control_points_mem_buffer = Some(create_buffer_init(
        &vulkan_base,
        teapot_data.get_control_points_slice(),
        vk::BufferUsageFlags::STORAGE_BUFFER,
        vk::AccessFlags::SHADER_READ,
        vk::PipelineStageFlags::VERTEX_SHADER,
        "control points buffer",
    )?);

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

        internal_state
            .control_points_mem_buffer
            .as_ref()
            .map(|mem_buffer| {
                vulkan_base
                    .allocator
                    .destroy_buffer(mem_buffer.buffer, &mem_buffer.allocation)
            });
    }
}

fn create_shader_module(
    vulkan_base: &VulkanBase,
    path: &std::path::Path,
    object_name: &str,
) -> Result<vk::ShaderModule, String> {
    let shader_module = vulkan::create_shader_module(&vulkan_base.device, &path)?;

    vulkan::set_debug_utils_object_name(
        &vulkan_base.debug_utils_loader,
        vulkan_base.device.handle(),
        shader_module,
        object_name,
    );

    Ok(shader_module)
}

/*fn create_buffer(
    vulkan_base: &VulkanBase,
    size: vk::DeviceSize,
    buffer_usage: vk::BufferUsageFlags,
    memory_usage: vk_mem::MemoryUsage,
    memory_flags: vk_mem::AllocationCreateFlags,
    object_name: &str,
) -> Result<vulkan::MemBuffer, String> {
    let mem_buffer = vulkan::create_buffer(
        &vulkan_base.allocator,
        size,
        buffer_usage,
        memory_usage,
        memory_flags,
    )
    .map_err(|_| format!("failed to allocate {}", object_name))?;

    vulkan::set_debug_utils_object_name(
        &vulkan_base.debug_utils_loader,
        vulkan_base.device.handle(),
        mem_buffer.buffer,
        object_name,
    );

    vulkan::set_debug_utils_object_name(
        &vulkan_base.debug_utils_loader,
        vulkan_base.device.handle(),
        mem_buffer.allocation_info.get_device_memory(),
        &format!("{} device memory", object_name),
    );

    Ok(mem_buffer)
}*/
