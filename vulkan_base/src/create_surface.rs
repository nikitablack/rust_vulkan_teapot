use ash::vk;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

pub fn create_surface(
    entry: &ash::Entry,
    instance: &ash::Instance,
    window: &winit::window::Window,
) -> Result<vk::SurfaceKHR, String> {
    log::info!("creating surface");

    let surface = unsafe {
        ash_window::create_surface(
            &entry,
            &instance,
            window.raw_display_handle(),
            window.raw_window_handle(),
            None,
        )
        .map_err(|_| String::from("failed to create surface"))?
    };

    log::info!("surface created");

    Ok(surface)
}
