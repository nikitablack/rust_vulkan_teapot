use ash::vk;

pub fn on_physical_device_change(
    vulkan_data: &mut crate::VulkanBaseData,
    window: &winit::window::Window,
    device_extensions: &Vec<&'static std::ffi::CStr>,
) -> crate::VulkanInitResult {
    // do not need to destroy swapchain now - the old swapchain is used when new swapchain is created
    let old_swapchain = vulkan_data.swapchain;
    vulkan_data.swapchain = vk::SwapchainKHR::null();

    crate::clear_vulkan_base(vulkan_data);

    vulkan_data.swapchain = old_swapchain;

    crate::create_logical_device(vulkan_data, device_extensions)?;
    crate::get_device_queue(vulkan_data);
    crate::create_allocator(vulkan_data)?;
    crate::create_swapchain_loader(vulkan_data);
    crate::rebuild_swapchain_data(vulkan_data, window)?;

    Ok(())
}
