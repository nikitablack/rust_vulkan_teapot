use ash::version::DeviceV1_0;

pub fn rebuild_swapchain_data(
    vulkan_data: &mut crate::VulkanBaseData,
    window: &winit::window::Window,
) -> crate::VulkanInitResult {
    let device = vulkan_data.get_device_ref();

    unsafe {
        for &view in &vulkan_data.swapchain_image_views {
            device.destroy_image_view(view, None);
        }
    }
    vulkan_data.swapchain_image_views.clear();

    crate::get_surface_capabilities(vulkan_data)?;
    crate::get_surface_extent(vulkan_data, window);
    crate::create_swapchain(vulkan_data)?;
    crate::get_swapchain_images(vulkan_data)?;
    crate::get_swapchain_image_views(vulkan_data)?;

    Ok(())
}
