mod check_instance_version;
mod check_required_instance_extensions;
mod clear_vulkan_base;
mod create_allocator;
mod create_debug_utils_loader;
mod create_instance;
mod create_logical_device;
mod create_surface;
mod create_surface_loader;
mod create_swapchain;
mod create_swapchain_loader;
mod get_device_queue;
mod get_physical_devices;
mod get_surface_capabilities;
mod get_surface_extent;
mod get_swapchain_image_views;
mod get_swapchain_images;
mod on_physical_device_change;
mod rebuild_swapchain_data;
mod vulkan_base_data;

pub use check_instance_version::*;
pub use check_required_instance_extensions::*;
pub use clear_vulkan_base::*;
pub use create_allocator::*;
pub use create_debug_utils_loader::*;
pub use create_instance::*;
pub use create_logical_device::*;
pub use create_surface::*;
pub use create_surface_loader::*;
pub use create_swapchain::*;
pub use create_swapchain_loader::*;
pub use get_device_queue::*;
pub use get_physical_devices::*;
pub use get_surface_capabilities::*;
pub use get_surface_extent::*;
pub use get_swapchain_image_views::*;
pub use get_swapchain_images::*;
pub use on_physical_device_change::*;
pub use rebuild_swapchain_data::*;
pub use vulkan_base_data::*;

fn init_internal(
    vulkan_data: &mut VulkanBaseData,
    window: &winit::window::Window,
    instance_extensions: &Vec<&'static std::ffi::CStr>,
    device_extensions: &Vec<&'static std::ffi::CStr>,
    enable_debug_utils: bool,
    device_index: usize,
) -> Result<(), String> {
    vulkan_data.entry = match ash::Entry::new() {
        Ok(entry) => Some(entry),
        Err(_) => return Err(String::from("failed to create Entry")),
    };

    check_instance_version(vulkan_data)?;
    check_required_instance_extensions(vulkan_data, instance_extensions)?;
    create_instance(vulkan_data, instance_extensions)?;

    if enable_debug_utils {
        create_debug_utils_loader(vulkan_data);
    }

    create_surface_loader(vulkan_data);
    create_surface(vulkan_data, window)?;
    get_physical_devices(vulkan_data, device_extensions)?;

    vulkan_data.selected_physical_device_index = device_index;
    if vulkan_data.selected_physical_device_index >= vulkan_data.physical_devices.len() {
        vulkan_data.selected_physical_device_index = 0;
    }

    log::info!(
        "selected physical device index: {}",
        vulkan_data.selected_physical_device_index
    );

    on_physical_device_change(vulkan_data, window, device_extensions)?;

    Ok(())
}

pub fn init_vulkan(
    window: &winit::window::Window,
    instance_extensions: &Vec<&'static std::ffi::CStr>,
    device_extensions: &Vec<&'static std::ffi::CStr>,
    enable_debug_utils: bool,
    device_index: usize,
) -> Result<VulkanBaseData, String> {
    let mut vulkan_data = VulkanBaseData::default();

    if let Err(msg) = init_internal(
        &mut vulkan_data,
        window,
        instance_extensions,
        device_extensions,
        enable_debug_utils,
        device_index,
    ) {
        clear_vulkan_base(&mut vulkan_data);
        return Err(msg);
    }

    Ok(vulkan_data)
}
