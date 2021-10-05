use ash::extensions::khr;

pub fn create_swapchain_loader(instance: &ash::Instance, device: &ash::Device) -> khr::Swapchain {
    let swapchain_loader = khr::Swapchain::new(instance, device);

    log::info!("swapchain loader created");

    swapchain_loader
}
