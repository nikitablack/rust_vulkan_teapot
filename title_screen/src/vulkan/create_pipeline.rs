use crate::vulkan;
use ash::version::DeviceV1_0;
use ash::vk;

pub fn create_pipeline(
    data: &mut vulkan::TitleScreenVulkanData,
    vulkan_base_data: &vulkan_base::VulkanBaseData,
) -> common::VulkanResult {
    let device = vulkan_base_data.get_device_ref();

    let shader_entry_name = std::ffi::CString::new("main").unwrap();

    let vs_state = vk::PipelineShaderStageCreateInfo {
        stage: vk::ShaderStageFlags::VERTEX,
        module: data.vertex_shader_module,
        p_name: shader_entry_name.as_ptr(),
        ..Default::default()
    };

    let fs_state = vk::PipelineShaderStageCreateInfo {
        stage: vk::ShaderStageFlags::FRAGMENT,
        module: data.fragment_shader_module,
        p_name: shader_entry_name.as_ptr(),
        ..Default::default()
    };

    let ia_state = vk::PipelineInputAssemblyStateCreateInfo {
        topology: vk::PrimitiveTopology::TRIANGLE_LIST,
        ..Default::default()
    };

    let raster_state = vk::PipelineRasterizationStateCreateInfo {
        polygon_mode: vk::PolygonMode::FILL,
        cull_mode: vk::CullModeFlags::NONE,
        front_face: vk::FrontFace::CLOCKWISE,
        line_width: 1.0f32,
        ..Default::default()
    };

    let col_blend_attachment_state = vk::PipelineColorBlendAttachmentState {
        blend_enable: vk::TRUE,
        src_color_blend_factor: vk::BlendFactor::SRC_ALPHA,
        dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
        color_blend_op: vk::BlendOp::ADD,
        src_alpha_blend_factor: vk::BlendFactor::SRC_ALPHA,
        dst_alpha_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
        alpha_blend_op: vk::BlendOp::ADD,
        color_write_mask: vk::ColorComponentFlags::R
            | vk::ColorComponentFlags::G
            | vk::ColorComponentFlags::B
            | vk::ColorComponentFlags::A,
    };

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

    let vert_inp_state = vk::PipelineVertexInputStateCreateInfo {
        ..Default::default()
    };

    let stages = [vs_state, fs_state];
    let create_info = vk::GraphicsPipelineCreateInfo::builder()
        .flags(vk::PipelineCreateFlags::empty())
        .stages(&stages)
        .input_assembly_state(&ia_state)
        .rasterization_state(&raster_state)
        .color_blend_state(&col_blend_state)
        .dynamic_state(&dyn_state)
        .layout(data.pipeline_layout)
        .render_pass(data.render_pass)
        .subpass(0)
        .viewport_state(&viewport_state)
        .multisample_state(&multisample_state)
        .vertex_input_state(&vert_inp_state)
        .build();

    let pipelines = match unsafe {
        device.create_graphics_pipelines(vk::PipelineCache::null(), &[create_info], None)
    } {
        Ok(p) => p,
        Err(_) => return Err(String::from("failed to create pipeline")),
    };

    data.pipeline = pipelines[0];

    if let Some(debug_utils) = vulkan_base_data.debug_utils_loader.as_ref() {
        common::set_debug_utils_object_name(
            debug_utils,
            device.handle(),
            data.pipeline,
            String::from("title screen pipeline"),
        );
    }

    Ok(())
}
