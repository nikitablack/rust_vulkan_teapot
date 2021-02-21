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

#[derive(Default)]
pub struct VulkanData {
    pub vertex_shader_module: vk::ShaderModule,
    pub tese_shader_module: vk::ShaderModule,
    pub tesc_shader_module: vk::ShaderModule,
    pub fragment_shader_module: vk::ShaderModule,
    pub control_points_mem_buffer: MemBuffer,
    pub patches_mem_buffer: MemBuffer,
    pub instances_mem_buffer: MemBuffer,
    pub uniform_mem_buffers: Vec<MemBuffer>,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub descriptor_pool: vk::DescriptorPool,
    pub pipeline_layout: vk::PipelineLayout,
    pub render_pass: vk::RenderPass,
    pub solid_pipeline: vk::Pipeline,
    pub wireframe_pipeline: vk::Pipeline,
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
                .destroy_descriptor_pool(self.descriptor_pool, None);

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

fn new_internal(vulkan_data: &mut VulkanData, vulkan_base: &VulkanBase) -> Result<(), String> {
    vulkan_data.vertex_shader_module = create_shader_module(
        vulkan_base,
        std::path::Path::new("shaders/shader.vert.spv"),
        "vertex shader",
    )?;

    vulkan_data.tese_shader_module = create_shader_module(
        vulkan_base,
        std::path::Path::new("shaders/shader.tese.spv"),
        "tesselation evaluation shader",
    )?;

    vulkan_data.tesc_shader_module = create_shader_module(
        vulkan_base,
        std::path::Path::new("shaders/shader.tesc.spv"),
        "tesselation control shader",
    )?;

    vulkan_data.fragment_shader_module = create_shader_module(
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

    vulkan_data.instances_mem_buffer = vulkan::create_buffer_init(
        vulkan_base,
        teapot_data.get_instances_slice(),
        vk::BufferUsageFlags::STORAGE_BUFFER,
        vk::AccessFlags::SHADER_READ,
        vk::PipelineStageFlags::TESSELLATION_EVALUATION_SHADER,
        "instances buffer",
    )?;

    for i in 0..crate::CONCURRENT_FRAME_COUNT {
        let buffer = vulkan::create_buffer(
            vulkan_base,
            (16 * std::mem::size_of::<f32>()) as vk::DeviceSize,
            vk::BufferUsageFlags::STORAGE_BUFFER,
            vk_mem::MemoryUsage::CpuToGpu,
            vk_mem::AllocationCreateFlags::MAPPED,
            &format!("uniform buffer {}", i),
        )?;

        vulkan_data.uniform_mem_buffers.push(buffer);
    }

    vulkan_data.descriptor_set_layout = create_descriptor_set_layout(vulkan_base)?;
    vulkan_data.descriptor_pool = create_descriptor_pool(vulkan_base)?;
    vulkan_data.pipeline_layout =
        create_pipeline_layout(vulkan_base, vulkan_data.descriptor_set_layout)?;
    vulkan_data.render_pass = create_render_pass(vulkan_base)?;
    let (solid_pipeline, wireframe_pipeline) = create_pipelines(
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

fn create_descriptor_set_layout(
    vulkan_base: &VulkanBase,
) -> Result<vk::DescriptorSetLayout, String> {
    let control_points_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(0)
        .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::VERTEX)
        .build();

    let patch_data_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(1)
        .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::TESSELLATION_EVALUATION)
        .build();

    let uniform_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(2)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::TESSELLATION_EVALUATION)
        .build();

    let bindings = [control_points_binding, patch_data_binding, uniform_binding];
    let create_info = vk::DescriptorSetLayoutCreateInfo::builder()
        .bindings(&bindings)
        .build();

    let descriptor_set_layout = unsafe {
        vulkan_base
            .device
            .create_descriptor_set_layout(&create_info, None)
            .map_err(|_| String::from("failed to create descriptor set layout"))?
    };

    vulkan::set_debug_utils_object_name(
        &vulkan_base.debug_utils_loader,
        vulkan_base.device.handle(),
        descriptor_set_layout,
        "descriptor set layout",
    );

    Ok(descriptor_set_layout)
}

fn create_descriptor_pool(vulkan_base: &VulkanBase) -> Result<vk::DescriptorPool, String> {
    let pool_size_1 = vk::DescriptorPoolSize {
        ty: vk::DescriptorType::STORAGE_BUFFER,
        descriptor_count: 100,
    };

    let pool_size_2 = vk::DescriptorPoolSize {
        ty: vk::DescriptorType::UNIFORM_BUFFER,
        descriptor_count: 100,
    };

    let sizes = [pool_size_1, pool_size_2];
    let create_info = vk::DescriptorPoolCreateInfo::builder()
        .max_sets(100)
        .pool_sizes(&sizes)
        .build();

    let descriptor_pool = unsafe {
        vulkan_base
            .device
            .create_descriptor_pool(&create_info, None)
            .map_err(|_| String::from("failed to create descriptor pool"))?
    };

    vulkan::set_debug_utils_object_name(
        &vulkan_base.debug_utils_loader,
        vulkan_base.device.handle(),
        descriptor_pool,
        "descriptor pool",
    );

    Ok(descriptor_pool)
}

fn create_pipeline_layout(
    vulkan_base: &VulkanBase,
    descriptor_set_layout: vk::DescriptorSetLayout,
) -> Result<vk::PipelineLayout, String> {
    let push_const_range = vk::PushConstantRange {
        stage_flags: vk::ShaderStageFlags::TESSELLATION_CONTROL,
        offset: 0,
        size: 4,
    };

    let laytouts = [descriptor_set_layout];
    let ranges = [push_const_range];
    let create_info = vk::PipelineLayoutCreateInfo::builder()
        .set_layouts(&laytouts)
        .push_constant_ranges(&ranges)
        .build();

    let pipeline_layout = unsafe {
        vulkan_base
            .device
            .create_pipeline_layout(&create_info, None)
            .map_err(|_| String::from("failed to create pipeline layout"))?
    };

    vulkan::set_debug_utils_object_name(
        &vulkan_base.debug_utils_loader,
        vulkan_base.device.handle(),
        pipeline_layout,
        "pipeline layout",
    );

    Ok(pipeline_layout)
}

fn create_render_pass(vulkan_base: &vulkan_base::VulkanBase) -> Result<vk::RenderPass, String> {
    let mut attachment_descriptions = Vec::new();

    attachment_descriptions.push(
        vk::AttachmentDescription::builder()
            .format(vulkan_base.surface_format.format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .build(),
    );

    let col_attachment_ref = vk::AttachmentReference::builder()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .build();

    let references = [col_attachment_ref];

    let mut subpass_descriptions = Vec::new();

    subpass_descriptions.push(
        vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&references)
            .build(),
    );

    let create_info = vk::RenderPassCreateInfo::builder()
        .attachments(&attachment_descriptions)
        .subpasses(&subpass_descriptions);

    let render_pass = unsafe {
        vulkan_base
            .device
            .create_render_pass(&create_info, None)
            .map_err(|_| String::from("failed to create render pass"))?
    };

    vulkan::set_debug_utils_object_name(
        &vulkan_base.debug_utils_loader,
        vulkan_base.device.handle(),
        render_pass,
        "render pass",
    );

    Ok(render_pass)
}

fn create_pipelines(
    vulkan_base: &vulkan_base::VulkanBase,
    vertex_shader_module: vk::ShaderModule,
    tess_control_shader_module: vk::ShaderModule,
    tess_eval_shader_module: vk::ShaderModule,
    fragment_shader_module: vk::ShaderModule,
    pipeline_layout: vk::PipelineLayout,
    render_pass: vk::RenderPass,
) -> Result<(vk::Pipeline, vk::Pipeline), String> {
    let shader_entry_name = std::ffi::CString::new("main").unwrap();

    let vs_state = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(vertex_shader_module)
        .name(&shader_entry_name)
        .build();

    let tc_state = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::TESSELLATION_CONTROL)
        .module(tess_control_shader_module)
        .name(&shader_entry_name)
        .build();

    let te_state = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::TESSELLATION_EVALUATION)
        .module(tess_eval_shader_module)
        .name(&shader_entry_name)
        .build();

    let fs_state = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(fragment_shader_module)
        .name(&shader_entry_name)
        .build();

    let ia_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::PATCH_LIST)
        .build();

    let raster_state = vk::PipelineRasterizationStateCreateInfo::builder()
        .polygon_mode(vk::PolygonMode::FILL)
        .cull_mode(vk::CullModeFlags::BACK)
        .front_face(vk::FrontFace::CLOCKWISE)
        .line_width(1.0f32)
        .build();

    let col_blend_attachment_state = vk::PipelineColorBlendAttachmentState::builder()
        .blend_enable(false)
        .build();

    let attachments = [col_blend_attachment_state];
    let col_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op(vk::LogicOp::CLEAR)
        .attachments(&attachments)
        .build();

    let states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let dyn_state = vk::PipelineDynamicStateCreateInfo::builder()
        .dynamic_states(&states)
        .build();

    let viewports = [vk::Viewport {
        ..Default::default()
    }];
    let scissors = [vk::Rect2D {
        ..Default::default()
    }];

    let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
        .viewports(&viewports)
        .scissors(&scissors)
        .build();

    let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
        .rasterization_samples(vk::SampleCountFlags::TYPE_1);

    let tessellation_state = vk::PipelineTessellationStateCreateInfo::builder()
        .patch_control_points(16)
        .build();

    let stages = [vs_state, tc_state, te_state, fs_state];

    let vert_inp_state = vk::PipelineVertexInputStateCreateInfo::builder().build();

    let solid_pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
        .flags(vk::PipelineCreateFlags::ALLOW_DERIVATIVES)
        .stages(&stages)
        .input_assembly_state(&ia_state)
        .rasterization_state(&raster_state)
        .color_blend_state(&col_blend_state)
        .dynamic_state(&dyn_state)
        .layout(pipeline_layout)
        .render_pass(render_pass)
        .subpass(0)
        .viewport_state(&viewport_state)
        .multisample_state(&multisample_state)
        .tessellation_state(&tessellation_state)
        .vertex_input_state(&vert_inp_state)
        .build();

    let raster_state = vk::PipelineRasterizationStateCreateInfo::builder()
        .polygon_mode(vk::PolygonMode::LINE)
        .cull_mode(vk::CullModeFlags::NONE)
        .front_face(vk::FrontFace::CLOCKWISE)
        .line_width(1.0f32)
        .build();

    let mut wireframe_pipeline_create_info = solid_pipeline_create_info;
    wireframe_pipeline_create_info.p_rasterization_state = &raster_state;
    wireframe_pipeline_create_info.base_pipeline_index = 0;

    let pipelines = unsafe {
        vulkan_base
            .device
            .create_graphics_pipelines(
                vk::PipelineCache::null(),
                &[solid_pipeline_create_info, wireframe_pipeline_create_info],
                None,
            )
            .map_err(|_| String::from("failed to create solid pipeline"))?
    };

    let solid_pipeline = pipelines[0];
    let wireframe_pipeline = pipelines[1];

    vulkan::set_debug_utils_object_name(
        &vulkan_base.debug_utils_loader,
        vulkan_base.device.handle(),
        solid_pipeline,
        "solid pipeline",
    );

    vulkan::set_debug_utils_object_name(
        &vulkan_base.debug_utils_loader,
        vulkan_base.device.handle(),
        wireframe_pipeline,
        "wireframe pipeline",
    );

    Ok((solid_pipeline, wireframe_pipeline))
}
