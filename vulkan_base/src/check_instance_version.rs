use ash::vk;

pub fn check_instance_version(entry: &ash::Entry) -> Result<(), String> {
    log::info!("checking instance version");

    let api_version = match entry.try_enumerate_instance_version() {
        Ok(result) => match result {
            Some(version) => version,
            None => vk::make_api_version(0, 1, 0, 0),
        },
        Err(_) => {
            return Err(String::from("failed to enumerate instance version"));
        }
    };

    log::info!(
        "instance version: {}.{}.{}",
        vk::api_version_major(api_version),
        vk::api_version_minor(api_version),
        vk::api_version_patch(api_version)
    );

    if vk::api_version_major(api_version) < 1 && vk::api_version_minor(api_version) < 2 {
        return Err(String::from(
            "minimum supported vulkan api version is 1.2.0",
        ));
    }

    Ok(())
}
