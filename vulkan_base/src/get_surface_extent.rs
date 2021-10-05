use ash::vk;

pub fn get_surface_extent(
    window: &winit::window::Window,
    surface_capabilities: &vk::SurfaceCapabilitiesKHR,
) -> vk::Extent2D {
    let window_size = window.inner_size();

    let mut surface_extent = vk::Extent2D::default();

    if surface_capabilities.current_extent.width == u32::MAX {
        surface_extent.width = std::cmp::max(
            surface_capabilities.min_image_extent.width,
            std::cmp::min(
                surface_capabilities.max_image_extent.width,
                window_size.width,
            ),
        );
        surface_extent.height = std::cmp::max(
            surface_capabilities.min_image_extent.height,
            std::cmp::min(
                surface_capabilities.max_image_extent.height,
                window_size.height,
            ),
        );
    } else {
        surface_extent = surface_capabilities.current_extent;
    }

    log::info!("surface extent got: {:?}", surface_extent);

    surface_extent
}
