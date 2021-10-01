use std::cell::RefCell;

use crate::{teapot_data, vulkan};
use ash::vk;
use scopeguard::{guard, ScopeGuard};
use vulkan_base::VulkanBase;

pub struct MemBuffer {
    pub buffer: vk::Buffer,
    pub size: vk::DeviceSize,
    pub allocation: gpu_allocator::vulkan::Allocation,
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
                log::warn!("tessellation evaluation shader scopeguard: destroying shader module");
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
                log::warn!("tessellation control shader scopeguard: destroying shader module");
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
                log::warn!("fragment shader scopeguard: destroying shader module");
                unsafe {
                    vulkan_base.device.destroy_shader_module(sm, None);
                }
            })
        };

        let teapot_data = teapot_data::TeapotData::new();

        let control_points_mem_buffer = vulkan::create_gpu_buffer_init(
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

        let control_points_mem_buffer_sg = guard(control_points_mem_buffer, |mem_buffer| {
            log::info!("something went wrong, destroying control points buffer");
            unsafe {
                device.destroy_buffer(mem_buffer.buffer, None);
            }
            let _ = allocator_rc.borrow_mut().free(mem_buffer.allocation);
        });

        let patches_mem_buffer = vulkan::create_gpu_buffer_init(
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

        let patches_mem_buffer_sg = guard(patches_mem_buffer, |mem_buffer| {
            log::info!("something went wrong, destroying patches buffer");
            unsafe {
                device.destroy_buffer(mem_buffer.buffer, None);
            }
            let _ = allocator_rc.borrow_mut().free(mem_buffer.allocation);
        });

        let patch_point_count = teapot_data.get_patch_point_count();

        let instances_mem_buffer = vulkan::create_gpu_buffer_init(
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

        let instances_mem_buffer_sg = guard(instances_mem_buffer, |mem_buffer| {
            log::info!("something went wrong, destroying instances buffer");
            unsafe {
                device.destroy_buffer(mem_buffer.buffer, None);
            }
            let _ = allocator_rc.borrow_mut().free(mem_buffer.allocation);
        });

        let mut uniform_mem_buffer_sgs =
            Vec::with_capacity(crate::CONCURRENT_RESOURCE_COUNT as usize);
        for i in 0..crate::CONCURRENT_RESOURCE_COUNT {
            let mem_buffer = vulkan::create_buffer(
                &vulkan_base.device,
                *allocator_rc.borrow_mut(),
                &vulkan_base.debug_utils_loader,
                (16 * std::mem::size_of::<f32>()) as vk::DeviceSize,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                gpu_allocator::MemoryLocation::CpuToGpu,
                &format!("uniform buffer {}", i),
            )?;

            let uniform_mem_buffer_sg = guard(mem_buffer, |mem_buffer| {
                log::info!("something went wrong, destroying uniform buffer");
                unsafe {
                    device.destroy_buffer(mem_buffer.buffer, None);
                }
                let _ = allocator_rc.borrow_mut().free(mem_buffer.allocation);
            });

            uniform_mem_buffer_sgs.push(uniform_mem_buffer_sg);
        }

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

    pub fn clean(self, vulkan_base: &VulkanBase) {
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
        }
    }
}
