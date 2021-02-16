mod check_instance_version;
mod check_required_instance_extensions;
mod create_instance;
mod create_logical_device;
mod get_depth_format;
mod get_physical_device;
mod get_present_mode;
mod get_queue_family;
mod get_surface_format;

use ash::extensions::khr;
use ash::version::DeviceV1_0;
use ash::version::InstanceV1_0;
use ash::vk;

use check_instance_version::*;
use check_required_instance_extensions::*;
use create_instance::*;
use create_logical_device::*;
use get_depth_format::*;
use get_physical_device::*;
use get_present_mode::*;
use get_queue_family::*;
use get_surface_format::*;

pub struct VulkanBase {
    pub entry: ash::Entry,
    pub instance: ash::Instance,
    pub surface_loader: khr::Surface,
    pub debug_utils_loader: Option<ash::extensions::ext::DebugUtils>,
    pub surface: vk::SurfaceKHR,
    pub physical_device: vk::PhysicalDevice,
    pub physical_device_properties: vk::PhysicalDeviceProperties,
    pub surface_format: vk::SurfaceFormatKHR,
    pub present_mode: vk::PresentModeKHR,
    pub depth_format: vk::Format,
    pub queue_family: u32,
    pub device: ash::Device,
    pub queue: vk::Queue,
}

impl VulkanBase {
    pub fn new<'a, 'b>(
        window: &winit::window::Window,
        required_instance_extensions: &Vec<&'a std::ffi::CStr>,
        required_device_extensions: &Vec<&'b std::ffi::CStr>,
        enable_debug_utils: bool,
    ) -> Result<Self, String> {
        let entry = match ash::Entry::new() {
            Ok(entry) => entry,
            Err(_) => return Err(String::from("failed to create Entry")),
        };

        check_instance_version(&entry)?;
        check_required_instance_extensions(&entry, required_instance_extensions)?;

        let instance = create_instance(&entry, required_instance_extensions)?;

        let debug_utils_loader = match enable_debug_utils {
            true => {
                log::info!("debug utils loader created");
                Some(ash::extensions::ext::DebugUtils::new(&entry, &instance))
            }
            false => None,
        };

        let surface_loader = ash::extensions::khr::Surface::new(&entry, &instance);

        let surface = unsafe {
            ash_window::create_surface(&entry, &instance, window, None)
                .map_err(|_| String::from("failed to create surface"))?
        };

        let physical_device = get_physical_device(&instance, &required_device_extensions)?;
        let physical_device_properties =
            unsafe { instance.get_physical_device_properties(physical_device) };
        let surface_format = get_surface_format(physical_device, &surface_loader, surface)?;
        let present_mode = get_present_mode(physical_device, &surface_loader, surface)?;
        let queue_family = get_queue_family(physical_device, &instance, &surface_loader, surface)?;
        let depth_format = get_depth_format(physical_device, &instance)?;

        let device_name =
            unsafe { std::ffi::CStr::from_ptr(physical_device_properties.device_name.as_ptr()) };

        log::info!("selected physical device {:?}", device_name);
        log::info!(
            "\tsupported api version: {}.{}.{}",
            vk::version_major(physical_device_properties.api_version),
            vk::version_minor(physical_device_properties.api_version),
            vk::version_patch(physical_device_properties.api_version)
        );
        log::info!(
            "\tdriver version: {}",
            physical_device_properties.driver_version
        );
        log::info!("\tsurface format: {:?}", surface_format);
        log::info!("\tpresent mode: {:?}", present_mode);
        log::info!("\tdepth format: {:?}", depth_format);
        log::info!("\tqueue family: {}", queue_family);

        let device = create_logical_device(
            &instance,
            physical_device,
            queue_family,
            &required_device_extensions,
        )?;

        let queue = unsafe { device.get_device_queue(queue_family, 0) };

        let vulkan_base = Self {
            entry,
            instance,
            debug_utils_loader,
            surface_loader,
            surface,
            physical_device,
            physical_device_properties,
            surface_format,
            present_mode,
            queue_family,
            depth_format,
            device,
            queue,
        };

        Ok(vulkan_base)
    }

    pub fn clean(&self) {
        log::info!("cleaning vulkan base");

        unsafe {
            self.device.destroy_device(None);
        }
    }
}
