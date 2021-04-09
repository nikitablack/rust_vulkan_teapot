use crate::{teapot_data, vulkan};
use ash::{version::DeviceV1_0, vk};
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

pub struct MemImage {
    pub image: vk::Image,
    pub view: vk::ImageView,
    pub extent: vk::Extent3D,
    pub allocation: vk_mem::Allocation,
}

impl Default for MemImage {
    fn default() -> Self {
        Self {
            image: vk::Image::null(),
            view: vk::ImageView::null(),
            extent: vk::Extent3D {
                width: 0,
                height: 0,
                depth: 0,
            },
            allocation: vk_mem::Allocation::null(),
        }
    }
}

#[derive(Default)]
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
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub pipeline_layout: vk::PipelineLayout,
    pub render_pass: vk::RenderPass,
    pub solid_pipeline: vk::Pipeline,
    pub wireframe_pipeline: vk::Pipeline,
    pub depth_buffer: MemImage,
    pub framebuffers: Vec<vk::Framebuffer>,
    pub should_resize: bool,
    pub image_available_semaphore: vk::Semaphore,
    pub rendering_finished_semaphore: vk::Semaphore,
    pub fences: Vec<vk::Fence>,
    pub command_pools: Vec<vk::CommandPool>,
    pub descriptor_pools: Vec<vk::DescriptorPool>,
    pub available_command_buffers: Vec<Vec<vk::CommandBuffer>>,
    pub used_command_buffers: Vec<Vec<vk::CommandBuffer>>,
    pub curr_resource_index: u32,
    pub is_wireframe_mode: bool,
    pub tesselation_level: f32,
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

    pub fn resize(&mut self, vulkan_base: &VulkanBase) -> Result<(), String> {
        unsafe {
            let _ = vulkan_base
                .allocator
                .destroy_image(self.depth_buffer.image, &self.depth_buffer.allocation);

            vulkan_base
                .device
                .destroy_image_view(self.depth_buffer.view, None);

            for &framebuffer in &self.framebuffers {
                vulkan_base.device.destroy_framebuffer(framebuffer, None);
            }
        }

        self.depth_buffer = vulkan::create_depth_buffer(vulkan_base)?;

        self.framebuffers = vulkan::create_framebuffers(
            vulkan_base,
            self.render_pass,
            vulkan_base.surface_extent,
            self.depth_buffer.view,
        )?;

        Ok(())
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

            let _ = vulkan_base
                .allocator
                .destroy_image(self.depth_buffer.image, &self.depth_buffer.allocation);

            vulkan_base
                .device
                .destroy_image_view(self.depth_buffer.view, None);

            for &framebuffer in &self.framebuffers {
                vulkan_base.device.destroy_framebuffer(framebuffer, None);
            }

            vulkan_base
                .device
                .destroy_semaphore(self.image_available_semaphore, None);

            vulkan_base
                .device
                .destroy_semaphore(self.rendering_finished_semaphore, None);

            for &fence in &self.fences {
                vulkan_base.device.destroy_fence(fence, None);
            }

            for &command_pool in &self.command_pools {
                vulkan_base.device.destroy_command_pool(command_pool, None);
            }

            for &descriptor_pool in &self.descriptor_pools {
                vulkan_base
                    .device
                    .destroy_descriptor_pool(descriptor_pool, None);
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
    vulkan_data.depth_buffer = vulkan::create_depth_buffer(vulkan_base)?;
    vulkan_data.framebuffers = vulkan::create_framebuffers(
        vulkan_base,
        vulkan_data.render_pass,
        vulkan_base.surface_extent,
        vulkan_data.depth_buffer.view,
    )?;
    vulkan_data.image_available_semaphore =
        vulkan::create_semaphore(vulkan_base, "image available semaphore")?;
    vulkan_data.rendering_finished_semaphore =
        vulkan::create_semaphore(vulkan_base, "rendering finished semaphore")?;

    vulkan_data.fences = vulkan::create_fences(vulkan_base)?;
    vulkan_data.command_pools = vulkan::create_command_pools(vulkan_base)?;
    vulkan_data.descriptor_pools = vulkan::create_descriptor_pools(vulkan_base)?;
    vulkan_data.available_command_buffers = vec![vec![]; crate::CONCURRENT_RESOURCE_COUNT as usize];
    vulkan_data.used_command_buffers = vec![vec![]; crate::CONCURRENT_RESOURCE_COUNT as usize];
    vulkan_data.tesselation_level = 1.0;

    Ok(())
}
