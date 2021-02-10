use crate::vulkan;
use ash::version::DeviceV1_0;
use ash::vk;

pub fn allocate_descriptor_sets(
    data: &mut vulkan::TitleScreenVulkanData,
    vulkan_base_data: &vulkan_base::VulkanBaseData,
) -> common::VulkanResult {
    debug_assert!(data.descriptor_sets.is_empty());

    let device_ref = vulkan_base_data.get_device_ref();

    let layouts = [data.descriptor_set_layout; common::NUM_RESOURCES_IN_FLIGHT as usize];

    let alloc_info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(data.descriptor_pool)
        .set_layouts(&layouts)
        .build();

    data.descriptor_sets = match unsafe { device_ref.allocate_descriptor_sets(&alloc_info) } {
        Ok(sets) => sets,
        Err(_) => return Err(String::from("failed to allocate descriptor sets")),
    };

    if let Some(debug_utils) = vulkan_base_data.debug_utils_loader.as_ref() {
        for (i, &set) in data.descriptor_sets.iter().enumerate() {
            common::set_debug_utils_object_name(
                debug_utils,
                device_ref.handle(),
                set,
                format!("title screen descriptor set {}", i),
            );
        }
    }

    Ok(())
}
