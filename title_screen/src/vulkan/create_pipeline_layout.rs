use crate::vulkan;
use ash::version::DeviceV1_0;
use ash::vk;

pub fn create_pipeline_layout(
    data: &mut vulkan::TitleScreenVulkanData,
    vulkan_base_data: &vulkan_base::VulkanBaseData,
) -> common::VulkanResult {
    let device = vulkan_base_data.get_device_ref();

    let pc_range = vk::PushConstantRange {
        stage_flags: vk::ShaderStageFlags::VERTEX,
        offset: 0,
        size: 4 * 4,
    };

    let laytouts = [data.descriptor_set_layout];
    let ranges = [pc_range];
    let create_info = vk::PipelineLayoutCreateInfo::builder()
        .set_layouts(&laytouts)
        .push_constant_ranges(&ranges)
        .build();

    data.pipeline_layout = match unsafe { device.create_pipeline_layout(&create_info, None) } {
        Ok(pl) => pl,
        Err(_) => return Err(String::from("failed to create pipeline layout")),
    };

    if let Some(debug_utils) = vulkan_base_data.debug_utils_loader.as_ref() {
        common::set_debug_utils_object_name(
            debug_utils,
            device.handle(),
            data.pipeline_layout,
            String::from("title screen pipeline layout"),
        );
    }

    Ok(())
}
