use std::cell::RefCell;

use crate::teapot_data;
use ash::vk;
use scopeguard::{guard, ScopeGuard};
use vulkan_base::VulkanBase;

pub struct VulkanData {
    pub vertex_shader_module: vk::ShaderModule,
    pub tese_shader_module: vk::ShaderModule,
    pub tesc_shader_module: vk::ShaderModule,
    pub fragment_shader_module: vk::ShaderModule,
    pub control_points_mem_buffer: vulkan_utils::MemBuffer,
    pub patches_mem_buffer: vulkan_utils::MemBuffer,
    pub patch_point_count: u32,
    pub instances_mem_buffer: vulkan_utils::MemBuffer,
    pub uniform_mem_buffers: Vec<vulkan_utils::MemBuffer>,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub pipeline_layout: vk::PipelineLayout,
    pub render_pass: vk::RenderPass,
    pub solid_pipeline: vk::Pipeline,
    pub wireframe_pipeline: vk::Pipeline,
}

impl VulkanData {
    pub fn new(vulkan_base: &mut VulkanBase) -> Result<Self, String> {
        let device = &vulkan_base.device;
        let allocator_rc = RefCell::new(&mut vulkan_base.allocator);

        let vertex_sm_sg = {
            let vertex_sm = vulkan_utils::create_shader_module(
                &vulkan_base.device,
                std::path::Path::new("shaders/shader.vert.spv"),
                &vulkan_base.debug_utils_loader,
                "vertex shader",
            )?;

            scopeguard::guard(vertex_sm, |sm| {
                log::warn!("vertex shader scopeguard: destroying shader module");
                unsafe {
                    device.destroy_shader_module(sm, None);
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
                log::warn!("tessellation evaluation shader scopeguard: destroying shader module");
                unsafe {
                    device.destroy_shader_module(sm, None);
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
                log::warn!("tessellation control shader scopeguard: destroying shader module");
                unsafe {
                    device.destroy_shader_module(sm, None);
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
                log::warn!("fragment shader scopeguard: destroying shader module");
                unsafe {
                    device.destroy_shader_module(sm, None);
                }
            })
        };

        let teapot_data = teapot_data::TeapotData::new();

        let control_points_mem_buffer_sg = {
            let control_points_mem_buffer = vulkan_utils::create_gpu_buffer_init(
                &vulkan_base.device,
                *allocator_rc.borrow_mut(),
                &vulkan_base.debug_utils_loader,
                vulkan_base.queue_family,
                vulkan_base.queue,
                teapot_data.get_control_points_slice(),
                vk::BufferUsageFlags::STORAGE_BUFFER,
                vk::AccessFlags::SHADER_READ,
                vk::PipelineStageFlags::VERTEX_SHADER,
                "control points buffer",
            )?;

            guard(control_points_mem_buffer, |mem_buffer| {
                log::warn!("control points scopeguard: destroying buffer and memory");
                unsafe {
                    device.destroy_buffer(mem_buffer.buffer, None);
                }
                let _ = allocator_rc.borrow_mut().free(mem_buffer.allocation);
            })
        };

        let patches_mem_buffer_sg = {
            let patches_mem_buffer = vulkan_utils::create_gpu_buffer_init(
                &vulkan_base.device,
                *allocator_rc.borrow_mut(),
                &vulkan_base.debug_utils_loader,
                vulkan_base.queue_family,
                vulkan_base.queue,
                teapot_data.get_patches_slice(),
                vk::BufferUsageFlags::INDEX_BUFFER,
                vk::AccessFlags::INDEX_READ,
                vk::PipelineStageFlags::VERTEX_INPUT,
                "patches buffer",
            )?;

            guard(patches_mem_buffer, |mem_buffer| {
                log::warn!("patches scopeguard: destroying buffer and memory");
                unsafe {
                    device.destroy_buffer(mem_buffer.buffer, None);
                }
                let _ = allocator_rc.borrow_mut().free(mem_buffer.allocation);
            })
        };

        let patch_point_count = teapot_data.get_patch_point_count();

        let instances_mem_buffer_sg = {
            let instances_mem_buffer = vulkan_utils::create_gpu_buffer_init(
                &vulkan_base.device,
                *allocator_rc.borrow_mut(),
                &vulkan_base.debug_utils_loader,
                vulkan_base.queue_family,
                vulkan_base.queue,
                teapot_data.get_instances_slice(),
                vk::BufferUsageFlags::STORAGE_BUFFER,
                vk::AccessFlags::SHADER_READ,
                vk::PipelineStageFlags::TESSELLATION_EVALUATION_SHADER,
                "instances buffer",
            )?;

            guard(instances_mem_buffer, |mem_buffer| {
                log::warn!("instances scopeguard: destroying buffer and memory");
                unsafe {
                    device.destroy_buffer(mem_buffer.buffer, None);
                }
                let _ = allocator_rc.borrow_mut().free(mem_buffer.allocation);
            })
        };

        let mut uniform_mem_buffer_sgs =
            Vec::with_capacity(crate::CONCURRENT_RESOURCE_COUNT as usize);
        for i in 0..crate::CONCURRENT_RESOURCE_COUNT {
            let mem_buffer = vulkan_utils::create_buffer(
                &vulkan_base.device,
                *allocator_rc.borrow_mut(),
                &vulkan_base.debug_utils_loader,
                (16 * std::mem::size_of::<f32>()) as vk::DeviceSize,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                gpu_allocator::MemoryLocation::CpuToGpu,
                &format!("uniform buffer {}", i),
            )?;

            let allocator_rc = &allocator_rc;
            let uniform_mem_buffer_sg = guard(mem_buffer, move |mem_buffer| {
                log::warn!(
                    "uniform buffer scopeguard {}: destroying buffer and memory",
                    i,
                );
                unsafe {
                    device.destroy_buffer(mem_buffer.buffer, None);
                }
                let _ = allocator_rc.borrow_mut().free(mem_buffer.allocation);
            });

            uniform_mem_buffer_sgs.push(uniform_mem_buffer_sg);
        }

        let descriptor_set_layout = vulkan::create_descriptor_set_layout(vulkan_base)?;
        let pipeline_layout =
            vulkan::create_pipeline_layout(vulkan_base, vulkan_data.descriptor_set_layout)?;
        let render_pass = vulkan::create_render_pass(vulkan_base)?;
        let (solid_pipeline, wireframe_pipeline) = vulkan::create_pipelines(
            vulkan_base,
            vulkan_data.vertex_shader_module,
            vulkan_data.tesc_shader_module,
            vulkan_data.tese_shader_module,
            vulkan_data.fragment_shader_module,
            vulkan_data.pipeline_layout,
            vulkan_data.render_pass,
        )?;

        Ok(VulkanData {
            vertex_shader_module: ScopeGuard::into_inner(vertex_sm_sg),
            tese_shader_module: ScopeGuard::into_inner(tese_sm_sg),
            tesc_shader_module: ScopeGuard::into_inner(tesc_sm_sg),
            fragment_shader_module: ScopeGuard::into_inner(fragment_sm_sg),
            control_points_mem_buffer: ScopeGuard::into_inner(control_points_mem_buffer_sg),
            patches_mem_buffer: ScopeGuard::into_inner(patches_mem_buffer_sg),
            patch_point_count,
            instances_mem_buffer: ScopeGuard::into_inner(instances_mem_buffer_sg),
            uniform_mem_buffers: uniform_mem_buffer_sgs
                .into_iter()
                .map(|sg| ScopeGuard::into_inner(sg))
                .collect(),
        })
    }

    pub fn clean(self, vulkan_base: &mut VulkanBase) {
        log::info!("cleaning vulkan data");

        unsafe {
            let device = &vulkan_base.device;
            let allocator = &mut vulkan_base.allocator;

            device.destroy_shader_module(self.vertex_shader_module, None);
            device.destroy_shader_module(self.tese_shader_module, None);
            device.destroy_shader_module(self.tesc_shader_module, None);
            device.destroy_shader_module(self.fragment_shader_module, None);

            device.destroy_buffer(self.control_points_mem_buffer.buffer, None);
            let _ = allocator.free(self.control_points_mem_buffer.allocation);

            device.destroy_buffer(self.patches_mem_buffer.buffer, None);
            let _ = allocator.free(self.patches_mem_buffer.allocation);

            device.destroy_buffer(self.instances_mem_buffer.buffer, None);
            let _ = allocator.free(self.instances_mem_buffer.allocation);

            for mem_buffer in self.uniform_mem_buffers {
                device.destroy_buffer(mem_buffer.buffer, None);
                let _ = allocator.free(mem_buffer.allocation);
            }

            vulkan_base
                .device
                .destroy_descriptor_set_layout(self.descriptor_set_layout, None);

            vulkan_base
                .device
                .destroy_pipeline_layout(self.pipeline_layout, None);

            vulkan_base
                .device
                .destroy_render_pass(self.render_pass, None);

            vulkan_base
                .device
                .destroy_pipeline(self.solid_pipeline, None);

            vulkan_base
                .device
                .destroy_pipeline(self.wireframe_pipeline, None);
        }
    }
}
/*
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

    vulkan_data.descriptor_set_layout = vulkan::create_descriptor_set_layout(vulkan_base)?;
    vulkan_data.pipeline_layout =
        vulkan::create_pipeline_layout(vulkan_base, vulkan_data.descriptor_set_layout)?;
    vulkan_data.render_pass = vulkan::create_render_pass(vulkan_base)?;
    let (solid_pipeline, wireframe_pipeline) = vulkan::create_pipelines(
        vulkan_base,
        vulkan_data.vertex_shader_module,
        vulkan_data.tesc_shader_module,
        vulkan_data.tese_shader_module,
        vulkan_data.fragment_shader_module,
        vulkan_data.pipeline_layout,
        vulkan_data.render_pass,
    )?;
    vulkan_data.solid_pipeline = solid_pipeline;
    vulkan_data.wireframe_pipeline = wireframe_pipeline;

    Ok(())
}
*/
