use ash::extensions::khr;
use ash::vk;

pub fn create_swapchain(
    old_swapchain: vk::SwapchainKHR,
    surface: vk::SurfaceKHR,
    surface_capabilities: &vk::SurfaceCapabilitiesKHR,
    surface_format: &vk::SurfaceFormatKHR,
    surface_extent: vk::Extent2D,
    present_mode: vk::PresentModeKHR,
    swapchain_loader: &khr::Swapchain,
) -> Result<vk::SwapchainKHR, String> {
    log::info!("creating swapchain");

    let mut image_count = std::cmp::max(surface_capabilities.min_image_count, 3);

    if surface_capabilities.max_image_count != 0 {
        image_count = std::cmp::min(image_count, surface_capabilities.max_image_count);
    }

    log::info!("requested swapchain image count: {}", image_count);

    let create_info = vk::SwapchainCreateInfoKHR::builder()
        .surface(surface)
        .min_image_count(image_count)
        .image_format(surface_format.format)
        .image_color_space(surface_format.color_space)
        .image_extent(surface_extent)
        .image_array_layers(1)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
        .pre_transform(surface_capabilities.current_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(present_mode)
        .clipped(true)
        .old_swapchain(old_swapchain)
        .build();

    let swapchain = unsafe {
        swapchain_loader
            .create_swapchain(&create_info, None)
            .map_err(|_| String::from("failed to create swapchain"))?
    };

    if old_swapchain != vk::SwapchainKHR::null() {
        unsafe { swapchain_loader.destroy_swapchain(old_swapchain, None) };

        log::info!("old swapchain destroyed");
    }

    log::info!("swapchain created");

    Ok(swapchain)
}
