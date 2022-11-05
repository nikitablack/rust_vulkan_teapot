use ash::vk;

pub fn check_required_instance_extensions<'a>(
    entry: &ash::Entry,
    required_instance_extensions: &Vec<&'a std::ffi::CStr>,
) -> Result<(), String> {
    log::info!(
        "checking required instance extensions: {:?}",
        required_instance_extensions
    );

    let supported_instance_extensions = match entry.enumerate_instance_extension_properties(None) {
        Ok(props) => props,
        Err(_) => {
            return Err(String::from(
                "failed to enumerate instance extension properies",
            ))
        }
    };

    let mut supported_instance_extensions_set = std::collections::HashSet::new();
    for vk::ExtensionProperties { extension_name, .. } in &supported_instance_extensions {
        supported_instance_extensions_set
            .insert(unsafe { std::ffi::CStr::from_ptr(extension_name.as_ptr()) });
    }

    for &extension_name in required_instance_extensions {
        if !supported_instance_extensions_set.contains(extension_name) {
            return Err(format!(
                "instance extension {:?} is not supported",
                extension_name
            ));
        }
    }

    log::info!("all extensions are supported",);

    Ok(())
}
