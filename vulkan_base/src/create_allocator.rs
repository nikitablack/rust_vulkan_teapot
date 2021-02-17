use ash::vk;

pub fn create_allocator(
    instance: &ash::Instance,
    device: &ash::Device,
    physical_device: vk::PhysicalDevice,
) -> Result<vk_mem::Allocator, String> {
    let create_info = vk_mem::AllocatorCreateInfo {
        physical_device: physical_device,
        device: device.clone(),
        instance: instance.clone(),
        flags: vk_mem::AllocatorCreateFlags::empty(),
        preferred_large_heap_block_size: 0,
        frame_in_use_count: 0,
        heap_size_limits: None,
    };

    vk_mem::Allocator::new(&create_info).map_err(|_| String::from("failed to create allocator"))
}
