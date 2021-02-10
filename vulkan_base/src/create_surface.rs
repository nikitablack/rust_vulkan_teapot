pub fn create_surface(
    vulkan_data: &mut crate::VulkanBaseData,
    window: &winit::window::Window,
) -> crate::VulkanInitResult {
    assert_eq!(vulkan_data.surface, ash::vk::SurfaceKHR::null());

    vulkan_data.surface = match unsafe {
        ash_window::create_surface(
            vulkan_data.get_entry_ref(),
            vulkan_data.get_instance_ref(),
            window,
            None,
        )
    } {
        Ok(surface) => surface,
        Err(_) => return Err(String::from("failed to create surface")),
    };

    Ok(())
}
