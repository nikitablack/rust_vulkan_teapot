use ash::vk;

pub fn create_pipelines(
    device: &ash::Device,
    vertex_shader_module: vk::ShaderModule,
    tess_control_shader_module: vk::ShaderModule,
    tess_eval_shader_module: vk::ShaderModule,
    fragment_shader_module: vk::ShaderModule,
    pipeline_layout: vk::PipelineLayout,
    render_pass: vk::RenderPass,
    debug_utils_loader: &ash::extensions::ext::DebugUtils,
) -> Result<(vk::Pipeline, vk::Pipeline), String> {
    log::info!("creating pipelines");

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
        .color_write_mask(
            vk::ColorComponentFlags::R
                | vk::ColorComponentFlags::G
                | vk::ColorComponentFlags::B
                | vk::ColorComponentFlags::A,
        )
        .build();

    let attachments = [col_blend_attachment_state];
    let col_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
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

    let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::builder()
        .depth_test_enable(true)
        .depth_write_enable(true)
        .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL)
        .build();

    let solid_pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
        .flags(vk::PipelineCreateFlags::ALLOW_DERIVATIVES)
        .stages(&stages)
        .input_assembly_state(&ia_state)
        .rasterization_state(&raster_state)
        .color_blend_state(&col_blend_state)
        .dynamic_state(&dyn_state)
        .viewport_state(&viewport_state)
        .layout(pipeline_layout)
        .render_pass(render_pass)
        .subpass(0)
        .multisample_state(&multisample_state)
        .tessellation_state(&tessellation_state)
        .vertex_input_state(&vert_inp_state)
        .depth_stencil_state(&depth_stencil_state)
        .build();

    let raster_state = vk::PipelineRasterizationStateCreateInfo::builder()
        .polygon_mode(vk::PolygonMode::LINE)
        .cull_mode(vk::CullModeFlags::NONE)
        .front_face(vk::FrontFace::CLOCKWISE)
        .line_width(1.0f32)
        .build();

    let mut wireframe_pipeline_create_info = solid_pipeline_create_info;
    wireframe_pipeline_create_info.flags = vk::PipelineCreateFlags::DERIVATIVE;
    wireframe_pipeline_create_info.p_rasterization_state = &raster_state;
    wireframe_pipeline_create_info.base_pipeline_index = 0;

    let pipelines = unsafe {
        device
            .create_graphics_pipelines(
                vk::PipelineCache::null(),
                &[solid_pipeline_create_info, wireframe_pipeline_create_info],
                None,
            )
            .map_err(|_| String::from("failed to create pipelines"))?
    };

    let solid_pipeline = pipelines[0];
    let wireframe_pipeline = pipelines[1];

    vulkan_utils::set_debug_utils_object_name(
        debug_utils_loader,
        device.handle(),
        solid_pipeline,
        "solid pipeline",
    );

    vulkan_utils::set_debug_utils_object_name(
        debug_utils_loader,
        device.handle(),
        wireframe_pipeline,
        "wireframe pipeline",
    );

    log::info!("pipelines created");

    Ok((solid_pipeline, wireframe_pipeline))
}
