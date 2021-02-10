use crate::vulkan;
use ash::version::DeviceV1_0;
use ash::vk;

pub fn clear_title_screen_data(
    data: &mut vulkan::TitleScreenVulkanData,
    vulkan_base_data: &vulkan_base::VulkanBaseData,
) {
    log::info!("clearing title screen data");

    let device_ref = vulkan_base_data.get_device_ref();
    let allocator_ref = vulkan_base_data.get_allocator_ref();

    unsafe {
        let _ = device_ref.device_wait_idle();

        device_ref.destroy_render_pass(data.render_pass, None);
        data.render_pass = vk::RenderPass::null();

        for &f in &data.framebuffers {
            device_ref.destroy_framebuffer(f, None);
        }
        data.framebuffers.clear();

        device_ref.destroy_shader_module(data.vertex_shader_module, None);
        data.vertex_shader_module = vk::ShaderModule::null();

        device_ref.destroy_shader_module(data.fragment_shader_module, None);
        data.fragment_shader_module = vk::ShaderModule::null();

        device_ref.destroy_command_pool(data.command_pool, None);
        data.command_pool = vk::CommandPool::null();

        for mem_buf in &data.vertex_mem_buffers {
            let _ = allocator_ref.destroy_buffer(mem_buf.buffer, &mem_buf.allocation);
        }
        data.vertex_mem_buffers.clear();

        for mem_buf in &data.index_mem_buffers {
            let _ = allocator_ref.destroy_buffer(mem_buf.buffer, &mem_buf.allocation);
        }
        data.index_mem_buffers.clear();

        device_ref.destroy_descriptor_pool(data.descriptor_pool, None);
        data.descriptor_pool = vk::DescriptorPool::null();

        device_ref.destroy_descriptor_set_layout(data.descriptor_set_layout, None);
        data.descriptor_set_layout = vk::DescriptorSetLayout::null();

        device_ref.destroy_pipeline_layout(data.pipeline_layout, None);
        data.pipeline_layout = vk::PipelineLayout::null();

        for &f in &data.fences {
            device_ref.destroy_fence(f, None);
        }
        data.fences.clear();

        device_ref.destroy_semaphore(data.image_available_semaphore, None);
        data.image_available_semaphore = vk::Semaphore::null();

        device_ref.destroy_semaphore(data.graphics_finished_semaphore, None);
        data.graphics_finished_semaphore = vk::Semaphore::null();

        device_ref.destroy_pipeline(data.pipeline, None);
        data.pipeline = vk::Pipeline::null();

        let _ =
            allocator_ref.destroy_image(data.font_mem_image.image, &data.font_mem_image.allocation);

        device_ref.destroy_image_view(data.font_mem_image.view, None);
        data.font_mem_image = Default::default();

        device_ref.destroy_sampler(data.sampler, None);
        data.sampler = vk::Sampler::null();
    }
}
