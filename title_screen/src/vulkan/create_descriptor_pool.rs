use crate::vulkan;
use ash::version::DeviceV1_0;
use ash::vk;

pub fn create_descriptor_pool(
    data: &mut vulkan::TitleScreenVulkanData,
    vulkan_base_data: &vulkan_base::VulkanBaseData,
) -> common::VulkanResult {
    let device = vulkan_base_data.get_device_ref();

    let pool_size_1 = vk::DescriptorPoolSize {
        ty: vk::DescriptorType::STORAGE_BUFFER,
        descriptor_count: common::NUM_RESOURCES_IN_FLIGHT,
    };

    let pool_size_2 = vk::DescriptorPoolSize {
        ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
        descriptor_count: common::NUM_RESOURCES_IN_FLIGHT,
    };

    let sizes = [pool_size_1, pool_size_2];
    let create_info = vk::DescriptorPoolCreateInfo::builder()
        .max_sets(10)
        .pool_sizes(&sizes)
        .build();

    data.descriptor_pool = match unsafe { device.create_descriptor_pool(&create_info, None) } {
        Ok(p) => p,
        Err(_) => return Err(String::from("failed to create descriptor pool")),
    };

    if let Some(debug_utils) = vulkan_base_data.debug_utils_loader.as_ref() {
        common::set_debug_utils_object_name(
            debug_utils,
            device.handle(),
            data.descriptor_pool,
            String::from("title screen descriptor pool"),
        );
    }

    Ok(())
}
