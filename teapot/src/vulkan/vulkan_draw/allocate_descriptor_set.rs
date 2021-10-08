use ash::vk;

use crate::vulkan::VulkanData;
use vulkan_base::VulkanBase;

pub fn allocate_descriptor_set(
    vulkan_data: &mut VulkanData,
    vulkan_base: &VulkanBase,
) -> Result<vk::DescriptorSet, String> {
    let layouts = [vulkan_data.descriptor_set_layout; 1];

    let alloc_info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(vulkan_data.descriptor_pools[vulkan_data.curr_resource_index as usize])
        .set_layouts(&layouts)
        .build();

    let descriptor_sets = match unsafe { vulkan_base.device.allocate_descriptor_sets(&alloc_info) }
    {
        Ok(sets) => sets,
        Err(_) => return Err(String::from("failed to allocate descriptor sets")),
    };

    let set = descriptor_sets[0];

    vulkan_utils::set_debug_utils_object_name(
        &vulkan_base.debug_utils_loader,
        vulkan_base.device.handle(),
        set,
        "descriptor set",
    );

    Ok(set)
}
