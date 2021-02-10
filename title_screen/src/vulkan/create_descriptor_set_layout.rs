use crate::vulkan;
use ash::version::DeviceV1_0;
use ash::vk;

pub fn create_descriptor_set_layout(
    data: &mut vulkan::TitleScreenVulkanData,
    vulkan_base_data: &vulkan_base::VulkanBaseData,
) -> common::VulkanResult {
    let device = vulkan_base_data.get_device_ref();
    let vertex_binding = vk::DescriptorSetLayoutBinding {
        binding: 0,
        descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
        descriptor_count: 1,
        stage_flags: vk::ShaderStageFlags::VERTEX,
        ..Default::default()
    };

    let samplers = [data.sampler];
    let font_image_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(1)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::FRAGMENT)
        .immutable_samplers(&samplers)
        .build();

    let bindings = [vertex_binding, font_image_binding];
    let create_info = vk::DescriptorSetLayoutCreateInfo::builder()
        .bindings(&bindings)
        .build();

    data.descriptor_set_layout =
        match unsafe { device.create_descriptor_set_layout(&create_info, None) } {
            Ok(layout) => layout,
            Err(_) => return Err(String::from("failed to create descriptor set layout")),
        };

    if let Some(debug_utils) = vulkan_base_data.debug_utils_loader.as_ref() {
        common::set_debug_utils_object_name(
            debug_utils,
            device.handle(),
            data.descriptor_set_layout,
            String::from("title screen descriptor set layout"),
        );
    }

    Ok(())
}
