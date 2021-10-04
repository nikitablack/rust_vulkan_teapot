use ash::extensions::khr;
use ash::vk;

pub fn get_swapchain_images(
    swapchain_loader: &khr::Swapchain,
    swapchain: vk::SwapchainKHR,
) -> Result<Vec<vk::Image>, String> {
    log::info!("getting swapchain images");

    let swapchain_images = unsafe {
        swapchain_loader
            .get_swapchain_images(swapchain)
            .map_err(|_| String::from("failed to get swapchain images"))?
    };

    log::info!("swapchain images got");
    log::info!("created swapchain image count: {}", swapchain_images.len());

    Ok(swapchain_images)
}
