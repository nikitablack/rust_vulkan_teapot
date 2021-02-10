use ash::version::EntryV1_0;
use ash::vk;

pub fn create_instance(
    vulkan_data: &mut crate::VulkanBaseData,
    instance_extensions: &Vec<&'static std::ffi::CStr>,
) -> crate::VulkanInitResult {
    assert!(vulkan_data.instance.is_none());

    let extension_names_raw = instance_extensions
        .iter()
        .map(|ext| ext.as_ptr())
        .collect::<Vec<_>>();

    let create_info =
        vk::InstanceCreateInfo::builder().enabled_extension_names(&extension_names_raw);

    vulkan_data.instance = match unsafe {
        vulkan_data
            .get_entry_ref()
            .create_instance(&create_info, None)
    } {
        Ok(instance) => Some(instance),
        Err(_) => return Err(String::from("failed to create instance")),
    };

    Ok(())
}
