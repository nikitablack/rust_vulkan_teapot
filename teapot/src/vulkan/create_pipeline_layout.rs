use crate::vulkan;
use ash::version::DeviceV1_0;
use ash::vk;

pub fn create_pipeline_layout(
    vulkan_base: &vulkan_base::VulkanBase,
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
