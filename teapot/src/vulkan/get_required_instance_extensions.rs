pub fn get_required_instance_extensions(
    window: &winit::window::Window,
    enable_debug_utils: bool,
) -> Result<Vec<&'static std::ffi::CStr>, String> {
    let mut instance_extensions = match ash_window::enumerate_required_extensions(window) {
        Ok(extensions) => extensions,
        Err(_) => {
            return Err(String::from(
                "failed to enumerate required instance extensions",
            ))
        }
    };

    if enable_debug_utils {
        instance_extensions.push(ash::extensions::ext::DebugUtils::name());
    }

    Ok(instance_extensions)
}
