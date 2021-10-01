use ash::vk;

pub fn create_instance<'a>(
    entry: &ash::Entry,
    instance_extensions: &Vec<&'a std::ffi::CStr>,
) -> Result<ash::Instance, String> {
    log::info!("creating instance");

    let extension_names_raw = instance_extensions
        .iter()
        .map(|ext| ext.as_ptr())
        .collect::<Vec<_>>();

    let app_info = vk::ApplicationInfo::builder()
        .api_version(vk::make_api_version(0, 1, 2, 0))
        .build();

    let create_info = vk::InstanceCreateInfo::builder()
        .enabled_extension_names(&extension_names_raw)
        .application_info(&app_info)
        .build();

    let instance = unsafe {
        entry
            .create_instance(&create_info, None)
            .map_err(|_| String::from("failed to create instance"))?
    };

    log::info!("instance created");

    Ok(instance)
}
