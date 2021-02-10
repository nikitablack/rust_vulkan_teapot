pub fn get_swapchain_images(vulkan_data: &mut crate::VulkanBaseData) -> crate::VulkanInitResult {
    vulkan_data.swapchain_images = match unsafe {
        vulkan_data
            .get_swapchain_loader_ref()
            .get_swapchain_images(vulkan_data.swapchain)
    } {
        Ok(images) => images,
        Err(_) => return Err(String::from("failed to get swapchain images")),
    };

    log::info!(
        "created swapchain image count: {}",
        vulkan_data.swapchain_images.len()
    );

    Ok(())
}
