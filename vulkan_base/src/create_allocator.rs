use ash::vk;
use gpu_allocator::vulkan;

pub fn create_allocator(
    instance: &ash::Instance,
    device: &ash::Device,
    physical_device: vk::PhysicalDevice,
) -> Result<vulkan::Allocator, String> {
    let debug_settings = gpu_allocator::AllocatorDebugSettings {
        log_memory_information: true,
        log_leaks_on_shutdown: true,
        store_stack_traces: false,
        log_allocations: true,
        log_frees: true,
        log_stack_traces: false,
    };

    let create_info = &vulkan::AllocatorCreateDesc {
        instance: instance.clone(),
        device: device.clone(),
        physical_device,
        debug_settings,
        buffer_device_address: false,
    };

    let allocator = vulkan::Allocator::new(&create_info)
        .map_err(|_| String::from("failed to create allocator"))?;

    log::info!("allocator created");

    Ok(allocator)
}
