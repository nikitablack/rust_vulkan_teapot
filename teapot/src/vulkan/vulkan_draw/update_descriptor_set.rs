use ash::vk;

use crate::vulkan::VulkanData;
use vulkan_base::VulkanBase;

pub fn update_descriptor_set(
    vulkan_data: &VulkanData,
    vulkan_base: &VulkanBase,
    set: vk::DescriptorSet,
) {
    let control_point_buffer_info = vk::DescriptorBufferInfo {
        buffer: vulkan_data.control_points_mem_buffer.buffer,
        offset: 0,
        range: vk::WHOLE_SIZE,
    };

    let instance_buffer_info = vk::DescriptorBufferInfo {
        buffer: vulkan_data.instances_mem_buffer.buffer,
        offset: 0,
        range: vk::WHOLE_SIZE,
    };

    let uniform_buffer_info = vk::DescriptorBufferInfo {
        buffer: vulkan_data.uniform_mem_buffers[vulkan_data.curr_resource_index as usize].buffer,
        offset: 0,
        range: vk::WHOLE_SIZE,
    };

    let infos_1 = [control_point_buffer_info];
    let write_descriptor_set_1 = vk::WriteDescriptorSet::builder()
        .dst_set(set)
        .dst_binding(0)
        .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
        .buffer_info(&infos_1)
        .build();

    let infos_2 = [instance_buffer_info];
    let write_descriptor_set_2 = vk::WriteDescriptorSet::builder()
        .dst_set(set)
        .dst_binding(1)
        .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
        .buffer_info(&infos_2)
        .build();

    let infos_3 = [uniform_buffer_info];
    let write_descriptor_set_3 = vk::WriteDescriptorSet::builder()
        .dst_set(set)
        .dst_binding(2)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .buffer_info(&infos_3)
        .build();

    unsafe {
        vulkan_base.device.update_descriptor_sets(
            &[
                write_descriptor_set_1,
                write_descriptor_set_2,
                write_descriptor_set_3,
            ],
            &[],
        );
    }
}
