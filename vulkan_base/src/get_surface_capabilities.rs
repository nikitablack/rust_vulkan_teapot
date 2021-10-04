pub use ash::extensions::khr;
pub use ash::vk;

pub fn get_surface_capabilities(
    surface_loader: &khr::Surface,
    physical_device: vk::PhysicalDevice,
    surface: vk::SurfaceKHR,
) -> Result<vk::SurfaceCapabilitiesKHR, String> {
    log::info!("getting surface capabilities");

    let surface_capabilities = unsafe {
        surface_loader
            .get_physical_device_surface_capabilities(physical_device, surface)
            .map_err(|_| String::from("failed to get physical device surface capabilities"))?
    };

    log::info!("surface capabilities got");

    Ok(surface_capabilities)
}
