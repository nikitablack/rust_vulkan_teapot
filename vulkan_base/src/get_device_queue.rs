use ash::version::DeviceV1_0;

pub fn get_device_queue(vulkan_data: &mut crate::VulkanBaseData) {
    let device_data = vulkan_data
        .physical_devices
        .get_mut(vulkan_data.selected_physical_device_index)
        .expect("physical device index is out of bounds");

    let device = vulkan_data
        .device
        .as_ref()
        .expect("device shouldn't be empty");

    device_data.queue = unsafe { device.get_device_queue(device_data.queue_family, 0) };
}
