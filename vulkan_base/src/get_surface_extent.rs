use ash::vk;

pub fn get_surface_extent(vulkan_data: &mut crate::VulkanBaseData, window: &winit::window::Window) {
    let window_size = window.inner_size();
    let capabilities_ref = &vulkan_data.surface_capabilities;

    let mut surface_extent = vk::Extent2D::default();
    if capabilities_ref.current_extent.width == u32::MAX {
        surface_extent.width = std::cmp::max(
            capabilities_ref.min_image_extent.width,
            std::cmp::min(capabilities_ref.max_image_extent.width, window_size.width),
        );
        surface_extent.height = std::cmp::max(
            capabilities_ref.min_image_extent.height,
            std::cmp::min(capabilities_ref.max_image_extent.height, window_size.height),
        );
    } else {
        surface_extent = capabilities_ref.current_extent;
    }

    vulkan_data.surface_extent = surface_extent;

    log::info!("surface extent: {:?}", vulkan_data.surface_extent);
}
