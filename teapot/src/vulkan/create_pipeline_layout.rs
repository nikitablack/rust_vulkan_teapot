use ash::vk;

pub fn create_pipeline_layout(
    device: &ash::Device,
    descriptor_set_layout: vk::DescriptorSetLayout,
    debug_utils_loader: &ash::extensions::ext::DebugUtils,
) -> Result<vk::PipelineLayout, String> {
    log::info!("creating pipeline layout");

    let push_const_range = vk::PushConstantRange {
        stage_flags: vk::ShaderStageFlags::TESSELLATION_CONTROL,
        offset: 0,
        size: 4,
    };

    let layouts = [descriptor_set_layout];
    let ranges = [push_const_range];
    let create_info = vk::PipelineLayoutCreateInfo::builder()
        .set_layouts(&layouts)
        .push_constant_ranges(&ranges)
        .build();

    let pipeline_layout = unsafe {
        device
            .create_pipeline_layout(&create_info, None)
            .map_err(|_| String::from("failed to create pipeline layout"))?
    };

    vulkan_utils::set_debug_utils_object_name(
        debug_utils_loader,
        device.handle(),
        pipeline_layout,
        "pipeline layout",
    );

    log::info!("pipeline layout created");

    Ok(pipeline_layout)
}
