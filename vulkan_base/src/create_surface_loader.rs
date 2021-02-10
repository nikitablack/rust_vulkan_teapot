pub fn create_surface_loader(vulkan_data: &mut crate::VulkanBaseData) {
    vulkan_data.surface_loader = Some(ash::extensions::khr::Surface::new(
        vulkan_data.get_entry_ref(),
        vulkan_data.get_instance_ref(),
    ));
}
