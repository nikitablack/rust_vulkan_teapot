use crate::vulkan;
use ash::vk;

pub fn copy_ui_data(
    data: &mut vulkan::TitleScreenVulkanData,
    vulkan_base_data: &vulkan_base::VulkanBaseData,
    ui_draw_data: &imgui::DrawData,
) -> common::VulkanResult {
    const VERTEX_DATA_SIZE: vk::DeviceSize = 20;
    const INDEX_DATA_SIZE: vk::DeviceSize = 2;

    const_assert_eq!(
        std::mem::size_of::<imgui::DrawVert>(),
        VERTEX_DATA_SIZE as usize
    );

    let allocator = vulkan_base_data.get_allocator_ref();

    let resource_index = data.resource_index as usize;

    {
        let vertex_mem_buffer = &data.vertex_mem_buffers[resource_index];
        let curr_vertex_buffer_size = vertex_mem_buffer.size;
        let new_vertex_buffer_data_size =
            (ui_draw_data.total_vtx_count as vk::DeviceSize) * VERTEX_DATA_SIZE;

        if new_vertex_buffer_data_size > curr_vertex_buffer_size {
            if let Err(_) =
                allocator.destroy_buffer(vertex_mem_buffer.buffer, &vertex_mem_buffer.allocation)
            {
                return Err(format!(
                    "failed to destroy vertex buffer {}",
                    resource_index
                ));
            }
            data.vertex_mem_buffers[resource_index] = vulkan::create_vertex_buffer(
                vulkan_base_data,
                new_vertex_buffer_data_size,
                format!("title screen vertex buffer {}", resource_index),
            )?;

            vulkan::update_descriptor_set(data, vulkan_base_data, resource_index);
        }
    }

    {
        let index_mem_buffer = &data.index_mem_buffers[resource_index];
        let curr_index_buffer_size = data.index_mem_buffers[data.resource_index as usize].size;
        let new_index_buffer_data_size =
            (ui_draw_data.total_idx_count as vk::DeviceSize) * INDEX_DATA_SIZE;

        if new_index_buffer_data_size > curr_index_buffer_size {
            if let Err(_) =
                allocator.destroy_buffer(index_mem_buffer.buffer, &index_mem_buffer.allocation)
            {
                return Err(format!("failed to destroy index buffer {}", resource_index));
            }
            data.index_mem_buffers[resource_index] = vulkan::create_index_buffer(
                vulkan_base_data,
                new_index_buffer_data_size,
                format!("title screen index buffer {}", resource_index),
            )?;

            vulkan::update_descriptor_set(data, vulkan_base_data, resource_index);
        }
    }

    let mut vertex_offset = 0isize;
    let mut index_offset = 0isize;
    for draw_list in ui_draw_data.draw_lists() {
        let vertex_data_raw = common::utils::t_to_u8(draw_list.vtx_buffer());
        let index_data_raw = common::utils::t_to_u8(draw_list.idx_buffer());

        unsafe {
            std::ptr::copy_nonoverlapping(
                vertex_data_raw.as_ptr(),
                data.vertex_mem_buffers[data.resource_index as usize]
                    .get_allocation_info_ref()
                    .get_mapped_data()
                    .offset(vertex_offset * 20),
                vertex_data_raw.len(),
            );

            std::ptr::copy_nonoverlapping(
                index_data_raw.as_ptr(),
                data.index_mem_buffers[data.resource_index as usize]
                    .get_allocation_info_ref()
                    .get_mapped_data()
                    .offset(index_offset * 2),
                index_data_raw.len(),
            );
        }

        vertex_offset += draw_list.vtx_buffer().len() as isize;
        index_offset += draw_list.idx_buffer().len() as isize;
    }

    Ok(())
}
