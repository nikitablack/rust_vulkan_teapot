pub fn create_allocator(vulkan_data: &mut crate::VulkanBaseData) -> crate::VulkanInitResult {
    debug_assert!(vulkan_data.allocator.is_none());

    let ref device_data = vulkan_data.physical_devices[vulkan_data.selected_physical_device_index];

    let create_info = vk_mem::AllocatorCreateInfo {
        physical_device: device_data.physical_device,
        device: vulkan_data.get_device_ref().clone(),
        instance: vulkan_data.get_instance_ref().clone(),
        flags: vk_mem::AllocatorCreateFlags::empty(),
        preferred_large_heap_block_size: 0,
        frame_in_use_count: 0,
        heap_size_limits: None,
    };

    vulkan_data.allocator = match vk_mem::Allocator::new(&create_info) {
        Ok(alloc) => Some(alloc),
        Err(_) => return Err(String::from("failed to create allocator")),
    };

    Ok(())
}
