use ash::extensions::khr;
use ash::vk;

pub fn get_present_mode(
    physical_device: vk::PhysicalDevice,
    surface_loader: &khr::Surface,
    surface: vk::SurfaceKHR,
) -> Result<vk::PresentModeKHR, String> {
    log::info!("getting present mode");

    let modes = match unsafe {
        surface_loader.get_physical_device_surface_present_modes(physical_device, surface)
    } {
        Ok(formats) => formats,
        Err(_) => {
            return Err(String::from(
                "failed to get physical device surface present modes",
            ));
        }
    };

    if modes.is_empty() {
        return Err(String::from(
            "failed to get physical device surface present modes",
        ));
    }

    if modes.contains(&vk::PresentModeKHR::MAILBOX) {
        let present_mode = vk::PresentModeKHR::MAILBOX;

        log::info!("selected present mode: {:?}", present_mode);

        return Ok(present_mode);
    }

    if modes.contains(&vk::PresentModeKHR::IMMEDIATE) {
        let present_mode = vk::PresentModeKHR::IMMEDIATE;

        log::info!("selected present mode: {:?}", present_mode);

        return Ok(present_mode);
    }

    let present_mode = vk::PresentModeKHR::FIFO;

    log::info!("selected present mode: {:?}", present_mode);

    Ok(present_mode)
}
