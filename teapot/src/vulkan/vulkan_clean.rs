pub fn vulkan_clean(
    vulkan_base: &mut Option<vulkan_base::VulkanBase>,
    vulkan_data: &mut Option<super::VulkanData>,
) {
    let mut vk_base = vulkan_base.take().unwrap();
    let vk_data = vulkan_data.take().unwrap();

    unsafe {
        let _ = vk_base.device.device_wait_idle();
    }

    vk_data.clean(&mut vk_base);
    vk_base.clean();
}
