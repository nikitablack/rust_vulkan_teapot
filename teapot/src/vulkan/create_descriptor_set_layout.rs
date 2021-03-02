use crate::vulkan;
use ash::version::DeviceV1_0;
use ash::vk;

pub fn create_descriptor_set_layout(
    vulkan_base: &vulkan_base::VulkanBase,
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
