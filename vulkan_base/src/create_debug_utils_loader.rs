pub fn create_debug_utils_loader(vulkan_data: &mut crate::VulkanBaseData) {
    vulkan_data.debug_utils_loader = Some(ash::extensions::ext::DebugUtils::new(
        vulkan_data.get_entry_ref(),
        vulkan_data.get_instance_ref(),
    ));

    log::info!("debug utils created")
}
