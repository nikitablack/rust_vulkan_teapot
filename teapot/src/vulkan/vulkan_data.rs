use crate::{teapot_data, vulkan};
use ash::vk;
use vulkan_base::VulkanBase;

pub struct MemBuffer {
    pub buffer: vk::Buffer,
    pub size: vk::DeviceSize,
    pub allocation: vk_mem::Allocation,
}

impl Default for MemBuffer {
    fn default() -> Self {
        Self {
            buffer: vk::Buffer::null(),
            size: 0,
            allocation: vk_mem::Allocation::null(),
        }
    }
}

pub struct VulkanData {
    pub vertex_shader_module: vk::ShaderModule,
    pub tese_shader_module: vk::ShaderModule,
    pub tesc_shader_module: vk::ShaderModule,
    pub fragment_shader_module: vk::ShaderModule,
    pub control_points_mem_buffer: MemBuffer,
    pub patches_mem_buffer: MemBuffer,
    pub patch_point_count: u32,
    pub instances_mem_buffer: MemBuffer,
    pub uniform_mem_buffers: Vec<MemBuffer>,
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

            let _ = vulkan_base.allocator.destroy_buffer(
                self.control_points_mem_buffer.buffer,
                &self.control_points_mem_buffer.allocation,
            );

            let _ = vulkan_base.allocator.destroy_buffer(
                self.patches_mem_buffer.buffer,
                &self.patches_mem_buffer.allocation,
            );

            let _ = vulkan_base.allocator.destroy_buffer(
                self.instances_mem_buffer.buffer,
                &self.instances_mem_buffer.allocation,
            );

            for mem_buffer in &self.uniform_mem_buffers {
                let _ = vulkan_base
                    .allocator
                    .destroy_buffer(mem_buffer.buffer, &mem_buffer.allocation);
            }
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

    let teapot_data = teapot_data::TeapotData::new();

    vulkan_data.control_points_mem_buffer = vulkan::create_buffer_init(
        vulkan_base,
        teapot_data.get_control_points_slice(),
        vk::BufferUsageFlags::STORAGE_BUFFER,
        vk::AccessFlags::SHADER_READ,
        vk::PipelineStageFlags::VERTEX_SHADER,
        "control points buffer",
    )?;

    vulkan_data.patches_mem_buffer = vulkan::create_buffer_init(
        vulkan_base,
        teapot_data.get_patches_slice(),
        vk::BufferUsageFlags::INDEX_BUFFER,
        vk::AccessFlags::INDEX_READ,
        vk::PipelineStageFlags::VERTEX_INPUT,
        "patches buffer",
    )?;

    vulkan_data.patch_point_count = teapot_data.get_patch_point_count();

    vulkan_data.instances_mem_buffer = vulkan::create_buffer_init(
        vulkan_base,
        teapot_data.get_instances_slice(),
        vk::BufferUsageFlags::STORAGE_BUFFER,
        vk::AccessFlags::SHADER_READ,
        vk::PipelineStageFlags::TESSELLATION_EVALUATION_SHADER,
        "instances buffer",
    )?;

    for i in 0..crate::CONCURRENT_RESOURCE_COUNT {
        let buffer = vulkan::create_buffer(
            vulkan_base,
            (16 * std::mem::size_of::<f32>()) as vk::DeviceSize,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk_mem::MemoryUsage::CpuToGpu,
            vk_mem::AllocationCreateFlags::MAPPED,
            &format!("uniform buffer {}", i),
        )?;

        vulkan_data.uniform_mem_buffers.push(buffer);
    }

    Ok(())
}
