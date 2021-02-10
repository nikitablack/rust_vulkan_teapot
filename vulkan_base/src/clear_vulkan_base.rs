use ash::version::DeviceV1_0;
use ash::vk;

pub fn clear_vulkan_base(mut vulkan_data: &mut crate::VulkanBaseData) {
    if vulkan_data.device.is_none() {
        return;
    }

    log::info!("clearing vulkan base data");

    unsafe {
        let _ = vulkan_data.get_device_ref().device_wait_idle();
    }

    clear_swapchain(&mut vulkan_data);

    unsafe {
        vulkan_data.get_allocator_mut().destroy();
        vulkan_data.allocator = None;
        vulkan_data.get_device_ref().destroy_device(None);
        vulkan_data.device = None;
    }
}

pub fn clear_swapchain(vulkan_data: &mut crate::VulkanBaseData) {
    unsafe {
        for &view in &vulkan_data.swapchain_image_views {
            vulkan_data.get_device_ref().destroy_image_view(view, None);
        }
        vulkan_data.swapchain_image_views.clear();

        vulkan_data
            .get_swapchain_loader_ref()
            .destroy_swapchain(vulkan_data.swapchain, None);
        vulkan_data.swapchain = vk::SwapchainKHR::null();
    }
}
