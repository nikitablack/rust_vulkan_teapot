use ash::vk;

pub fn get_queue(device: &ash::Device, queue_family: u32) -> vk::Queue {
    unsafe { device.get_device_queue(queue_family, 0) }
}
