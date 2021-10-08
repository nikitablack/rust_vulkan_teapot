use ash::vk;

pub fn get_physical_device_properties(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
) -> vk::PhysicalDeviceProperties {
    unsafe { instance.get_physical_device_properties(physical_device) }
}
