pub use ash::extensions::khr;
pub use ash::vk;

pub fn get_surface_capabilities(
    vulkan_data: &mut crate::VulkanBaseData,
) -> crate::VulkanInitResult {
    let ref device_data = vulkan_data.physical_devices[vulkan_data.selected_physical_device_index];

    vulkan_data.surface_capabilities = match unsafe {
        vulkan_data
            .get_surface_loader_ref()
            .get_physical_device_surface_capabilities(
                device_data.physical_device,
                vulkan_data.surface,
            )
    } {
        Ok(capabilities) => capabilities,
        Err(_) => {
            return Err(String::from(
                "failed to get physical device surface capabilities",
            ))
        }
    };

    Ok(())
}
