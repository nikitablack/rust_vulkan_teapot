use ash::vk;

pub fn get_queue(device: &ash::Device, queue_family: u32) -> vk::Queue {
    let queue = unsafe { device.get_device_queue(queue_family, 0) };

    log::info!("queue got");

    queue
}
