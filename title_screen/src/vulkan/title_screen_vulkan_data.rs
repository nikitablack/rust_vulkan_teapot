use ash::vk;

#[derive(Default)]
pub struct TitleScreenVulkanData {
    pub render_pass: vk::RenderPass,
    pub framebuffers: Vec<vk::Framebuffer>,
    pub vertex_shader_module: vk::ShaderModule,
    pub fragment_shader_module: vk::ShaderModule,
    pub command_pool: vk::CommandPool,
    pub vertex_mem_buffers: Vec<common::MemBuffer>,
    pub index_mem_buffers: Vec<common::MemBuffer>,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub pipeline_layout: vk::PipelineLayout,
    pub fences: Vec<vk::Fence>,
    pub image_available_semaphore: vk::Semaphore,
    pub graphics_finished_semaphore: vk::Semaphore,
    pub pipeline: vk::Pipeline,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
    pub resource_index: u32,
    pub command_buffers: Vec<vk::CommandBuffer>,
    pub font_mem_image: common::MemImage,
    pub sampler: vk::Sampler,
}
