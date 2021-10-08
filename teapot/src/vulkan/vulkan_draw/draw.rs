use crate::VulkanData;
use ash::vk;
use cgmath::{num_traits::ToPrimitive, perspective, Deg, Matrix4, Point3, Vector3};
use vulkan_base::VulkanBase;

pub fn draw(
    vulkan_data: &mut VulkanData,
    vulkan_base: &VulkanBase,
    time_since_biginning_sec: f32,
) -> Result<(), String> {
    let get_image_index_result = super::get_image_index(vulkan_data, vulkan_base)?;

    let image_index = match get_image_index_result {
        super::GetImageIndexResult::Index(index) => index,
        super::GetImageIndexResult::ShouldRebuildSwapchain => {
            println!("swapchain is suboptimal or out of date");
            vulkan_data.should_resize = true;
            return Ok(());
        }
    };

    super::wait_resource_available(vulkan_data, vulkan_base)?;
    super::reset_command_pool(vulkan_data, vulkan_base)?;
    let command_buffer = super::get_command_buffer(vulkan_data, vulkan_base)?;
    super::begin_command_buffer(vulkan_base, command_buffer)?;

    super::begin_render_pass(
        vulkan_data,
        vulkan_base,
        image_index as usize,
        command_buffer,
    );

    super::set_viewport(vulkan_base, command_buffer);
    super::set_scissor(vulkan_base, command_buffer);
    super::reset_descriptor_pool(vulkan_data, vulkan_base)?;
    let descriptor_set = super::allocate_descriptor_set(vulkan_data, vulkan_base)?;
    super::update_descriptor_set(vulkan_data, vulkan_base, descriptor_set);

    let model: Matrix4<f32> = Matrix4::from_translation(Vector3::new(0.0, 1.0, 0.0))
        * Matrix4::from_angle_x(Deg::<f32>(120.0))
        * Matrix4::from_angle_z(Deg::<f32>(time_since_biginning_sec * 20.0));

    let view = Matrix4::look_at_rh(
        Point3::<f32>::new(0.0, 0.0, -10.0),
        Point3::<f32>::new(0.0, 0.0, 0.0),
        Vector3::<f32>::new(0.0, 1.0, 0.0),
    );

    let projection = perspective(
        Deg::<f32>(45.0),
        vulkan_base
            .surface_extent
            .width
            .to_f32()
            .expect("failed to convert surface width to f32")
            / vulkan_base
                .surface_extent
                .height
                .to_f32()
                .expect("failed to convert surface width to f32"),
        0.1,
        100.0,
    );

    let mvp = projection * view * model;

    let curr_uniform_buffer =
        &mut vulkan_data.uniform_mem_buffers[vulkan_data.curr_resource_index as usize];

    let mvp_data = cgmath::conv::array4(mvp);
    let mvp_data_bytes = bytemuck::cast_slice(&mvp_data);

    curr_uniform_buffer.allocation.mapped_slice_mut().unwrap()[..16 * 4]
        .copy_from_slice(mvp_data_bytes);

    unsafe {
        vulkan_base.device.cmd_push_constants(
            command_buffer,
            vulkan_data.pipeline_layout,
            vk::ShaderStageFlags::TESSELLATION_CONTROL,
            0,
            bytemuck::cast_slice(&[vulkan_data.tesselation_level]),
        );

        vulkan_base.device.cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            vulkan_data.pipeline_layout,
            0,
            &[descriptor_set],
            &[],
        );

        let curr_pipeline = match vulkan_data.is_wireframe_mode {
            true => vulkan_data.wireframe_pipeline,
            false => vulkan_data.solid_pipeline,
        };

        vulkan_base.device.cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            curr_pipeline,
        );

        vulkan_base.device.cmd_bind_index_buffer(
            command_buffer,
            vulkan_data.patches_mem_buffer.buffer,
            0,
            vk::IndexType::UINT16,
        );

        vulkan_base.device.cmd_draw_indexed(
            command_buffer,
            vulkan_data.patch_point_count,
            1,
            0,
            0,
            0,
        );
    }

    unsafe {
        vulkan_base.device.cmd_end_render_pass(command_buffer);

        vulkan_base
            .device
            .end_command_buffer(command_buffer)
            .map_err(|_| String::from("failed to end command buffer"))?
    }

    super::submit(vulkan_data, vulkan_base, command_buffer)?;

    if !super::present(vulkan_data, vulkan_base, image_index)? {
        println!("swapchain is suboptimal or out of date");
        vulkan_data.should_resize = true;
        return Ok(());
    }

    Ok(())
}
