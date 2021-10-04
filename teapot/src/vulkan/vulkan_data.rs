use std::cell::RefCell;

use crate::teapot_data;
use crate::vulkan;
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
    pub framebuffers: Vec<vk::Framebuffer>,
    pub should_resize: bool,
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
                log::warn!("vertex shader scopeguard");
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
                log::warn!("tessellation evaluation shader scopeguard");
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
                log::warn!("tessellation control shader scopeguard");
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
                log::warn!("fragment shader scopeguard");
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
                log::warn!("control points buffer scopeguard");
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
                log::warn!("patches buffer scopeguard");
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
                log::warn!("instances buffer scopeguard");
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
                log::warn!("uniform buffer {} scopeguard", i);
                unsafe {
                    device.destroy_buffer(mem_buffer.buffer, None);
                }
                let _ = allocator_rc.borrow_mut().free(mem_buffer.allocation);
            });

            uniform_mem_buffer_sgs.push(uniform_mem_buffer_sg);
        }

        let descriptor_set_layout_sg = {
            let descriptor_set_layout = vulkan::create_descriptor_set_layout(
                &vulkan_base.device,
                &vulkan_base.debug_utils_loader,
            )?;

            guard(descriptor_set_layout, |layout| {
                log::warn!("descriptor set layout scopeguard");
                unsafe {
                    device.destroy_descriptor_set_layout(layout, None);
                }
            })
        };

        let pipeline_layout_sg = {
            let pipeline_layout = vulkan::create_pipeline_layout(
                &vulkan_base.device,
                *descriptor_set_layout_sg,
                &vulkan_base.debug_utils_loader,
            )?;

            guard(pipeline_layout, |layout| {
                log::warn!("pipeline layout scopeguard");
                unsafe {
                    device.destroy_pipeline_layout(layout, None);
                }
            })
        };

        let render_pass_sg = {
            let render_pass = vulkan::create_render_pass(
                &vulkan_base.device,
                vulkan_base.surface_format.format,
                &vulkan_base.debug_utils_loader,
            )?;

            guard(render_pass, |render_pass| {
                log::warn!("render pass scopeguard");
                unsafe {
                    device.destroy_render_pass(render_pass, None);
                }
            })
        };

        let (solid_pipeline_sg, wireframe_pipeline_sg) = {
            let (solid_pipeline, wireframe_pipeline) = vulkan::create_pipelines(
                &vulkan_base.device,
                *vertex_sm_sg,
                *tesc_sm_sg,
                *tese_sm_sg,
                *fragment_sm_sg,
                *pipeline_layout_sg,
                *render_pass_sg,
                &vulkan_base.debug_utils_loader,
            )?;

            let sg_1 = guard(solid_pipeline, |pipeline| {
                log::warn!("solid pipeline scopeguard");
                unsafe {
                    device.destroy_pipeline(pipeline, None);
                }
            });

            let sg_2 = guard(wireframe_pipeline, |pipeline| {
                log::warn!("wireframe pipeline scopeguard");
                unsafe {
                    device.destroy_pipeline(pipeline, None);
                }
            });

            (sg_1, sg_2)
        };

        vulkan_data.framebuffers = vulkan::create_framebuffers(
            vulkan_base,
            vulkan_data.render_pass,
            vulkan_base.surface_extent,
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
            descriptor_set_layout: ScopeGuard::into_inner(descriptor_set_layout_sg),
            pipeline_layout: ScopeGuard::into_inner(pipeline_layout_sg),
            render_pass: ScopeGuard::into_inner(render_pass_sg),
            solid_pipeline: ScopeGuard::into_inner(solid_pipeline_sg),
            wireframe_pipeline: ScopeGuard::into_inner(wireframe_pipeline_sg),
        })
    }

    pub fn resize(&mut self, vulkan_base: &VulkanBase) -> Result<(), String> {
        unsafe {
            for &framebuffer in &self.framebuffers {
                vulkan_base.device.destroy_framebuffer(framebuffer, None);
            }
        }

        self.framebuffers =
            vulkan::create_framebuffers(vulkan_base, self.render_pass, vulkan_base.surface_extent)?;

        Ok(())
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

            for &framebuffer in &self.framebuffers {
                vulkan_base.device.destroy_framebuffer(framebuffer, None);
            }
        }
    }
}
