use ash::vk;

pub fn create_surface(
    entry: &ash::Entry,
    instance: &ash::Instance,
    window: &winit::window::Window,
) -> Result<vk::SurfaceKHR, String> {
    let surface = unsafe {
        ash_window::create_surface(&entry, &instance, window, None)
            .map_err(|_| String::from("failed to create surface"))?
    };

    log::info!("surface created");

    Ok(surface)
}
