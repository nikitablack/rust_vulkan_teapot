use ash::vk;

pub fn create_descriptor_set_layout(
    device: &ash::Device,
    debug_utils_loader: &ash::extensions::ext::DebugUtils,
) -> Result<vk::DescriptorSetLayout, String> {
    log::info!("creating descriptor set layout");

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
        device
            .create_descriptor_set_layout(&create_info, None)
            .map_err(|_| String::from("failed to create descriptor set layout"))?
    };

    vulkan_utils::set_debug_utils_object_name(
        debug_utils_loader,
        device.handle(),
        descriptor_set_layout,
        "descriptor set layout",
    );

    log::info!("descriptor set layout created");

    Ok(descriptor_set_layout)
}
