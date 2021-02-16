use ash::version::EntryV1_0;
use ash::vk;

pub fn create_instance<'a>(
    entry: &ash::Entry,
    instance_extensions: &Vec<&'a std::ffi::CStr>,
) -> Result<ash::Instance, String> {
    let extension_names_raw = instance_extensions
        .iter()
        .map(|ext| ext.as_ptr())
        .collect::<Vec<_>>();

    let create_info =
        vk::InstanceCreateInfo::builder().enabled_extension_names(&extension_names_raw);

    unsafe {
        entry
            .create_instance(&create_info, None)
            .map_err(|_| String::from("failed to create instance"))
    }
}
