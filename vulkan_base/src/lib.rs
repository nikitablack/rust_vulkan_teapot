mod check_instance_version;
mod check_required_instance_extensions;
mod create_allocator;
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
use create_allocator::*;
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
    pub allocator: vk_mem::Allocator,
}

#[derive(Default)]
struct InternalState {
    entry: Option<ash::Entry>,
    instance: Option<ash::Instance>,
    surface_loader: Option<khr::Surface>,
    debug_utils_loader: Option<ash::extensions::ext::DebugUtils>,
    surface: vk::SurfaceKHR,
    physical_device: vk::PhysicalDevice,
    physical_device_properties: vk::PhysicalDeviceProperties,
    surface_format: vk::SurfaceFormatKHR,
    present_mode: vk::PresentModeKHR,
    depth_format: vk::Format,
    queue_family: u32,
    device: Option<ash::Device>,
    queue: vk::Queue,
    allocator: Option<vk_mem::Allocator>,
}

impl VulkanBase {
    pub fn new<'a, 'b>(
        window: &winit::window::Window,
        required_instance_extensions: &Vec<&'a std::ffi::CStr>,
        required_device_extensions: &Vec<&'b std::ffi::CStr>,
        enable_debug_utils: bool,
    ) -> Result<Self, String> {
        let mut internal_state = InternalState::default();

        if let Err(msg) = new_internal(
            &mut internal_state,
            window,
            required_instance_extensions,
            required_device_extensions,
            enable_debug_utils,
        ) {
            clean_internal(&mut internal_state);

            return Err(msg);
        }

        let vulkan_base = Self {
            entry: internal_state.entry.unwrap(),
            instance: internal_state.instance.unwrap(),
            debug_utils_loader: internal_state.debug_utils_loader,
            surface_loader: internal_state.surface_loader.unwrap(),
            surface: internal_state.surface,
            physical_device: internal_state.physical_device,
            physical_device_properties: internal_state.physical_device_properties,
            surface_format: internal_state.surface_format,
            present_mode: internal_state.present_mode,
            queue_family: internal_state.queue_family,
            depth_format: internal_state.depth_format,
            device: internal_state.device.unwrap(),
            queue: internal_state.queue,
            allocator: internal_state.allocator.unwrap(),
        };

        Ok(vulkan_base)
    }

    pub fn clean(&mut self) {
        log::info!("cleaning vulkan base");

        let mut internal_state = InternalState {
            entry: Some(self.entry.clone()),
            instance: Some(self.instance.clone()),
            surface_loader: Some(self.surface_loader.clone()),
            debug_utils_loader: self.debug_utils_loader.clone(),
            surface: self.surface,
            physical_device: self.physical_device,
            physical_device_properties: self.physical_device_properties,
            surface_format: self.surface_format,
            present_mode: self.present_mode,
            depth_format: self.depth_format,
            queue_family: self.queue_family,
            device: Some(self.device.clone()),
            queue: self.queue,
            allocator: None, //Some(self.allocator),
        };

        clean_internal(&mut internal_state);
    }
}

fn new_internal<'a, 'b>(
    state: &mut InternalState,
    window: &winit::window::Window,
    required_instance_extensions: &Vec<&'a std::ffi::CStr>,
    required_device_extensions: &Vec<&'b std::ffi::CStr>,
    enable_debug_utils: bool,
) -> Result<(), String> {
    state.entry = Some(ash::Entry::new().map_err(|_| String::from("failed to create Entry"))?);
    let entry = state.entry.as_ref().unwrap();

    check_instance_version(entry)?;
    check_required_instance_extensions(&entry, required_instance_extensions)?;

    state.instance = Some(create_instance(&entry, required_instance_extensions)?);
    let instance = state.instance.as_ref().unwrap();

    state.debug_utils_loader = match enable_debug_utils {
        true => {
            log::info!("debug utils loader created");
            Some(ash::extensions::ext::DebugUtils::new(entry, instance))
        }
        false => None,
    };

    state.surface_loader = Some(ash::extensions::khr::Surface::new(entry, instance));
    let surface_loader = state.surface_loader.as_ref().unwrap();

    state.surface = unsafe {
        ash_window::create_surface(entry, instance, window, None)
            .map_err(|_| String::from("failed to create surface"))?
    };

    state.physical_device = get_physical_device(&instance, &required_device_extensions)?;
    state.physical_device_properties =
        unsafe { instance.get_physical_device_properties(state.physical_device) };
    state.surface_format =
        get_surface_format(state.physical_device, surface_loader, state.surface)?;
    state.present_mode = get_present_mode(state.physical_device, surface_loader, state.surface)?;
    state.queue_family = get_queue_family(
        state.physical_device,
        instance,
        surface_loader,
        state.surface,
    )?;
    state.depth_format = get_depth_format(state.physical_device, instance)?;

    let device_name =
        unsafe { std::ffi::CStr::from_ptr(state.physical_device_properties.device_name.as_ptr()) };

    log::info!("selected physical device {:?}", device_name);
    log::info!(
        "\tsupported api version: {}.{}.{}",
        vk::version_major(state.physical_device_properties.api_version),
        vk::version_minor(state.physical_device_properties.api_version),
        vk::version_patch(state.physical_device_properties.api_version)
    );
    log::info!(
        "\tdriver version: {}",
        state.physical_device_properties.driver_version
    );
    log::info!("\tsurface format: {:?}", state.surface_format);
    log::info!("\tpresent mode: {:?}", state.present_mode);
    log::info!("\tdepth format: {:?}", state.depth_format);
    log::info!("\tqueue family: {}", state.queue_family);

    state.device = Some(create_logical_device(
        instance,
        state.physical_device,
        state.queue_family,
        &required_device_extensions,
    )?);
    let device = state.device.as_ref().unwrap();

    state.queue = unsafe { device.get_device_queue(state.queue_family, 0) };
    state.allocator = Some(create_allocator(&instance, device, state.physical_device)?);

    Ok(())
}

fn clean_internal(state: &mut InternalState) {
    unsafe {
        state
            .allocator
            .as_mut()
            .map(|allocator| allocator.destroy());
        state
            .device
            .as_ref()
            .map(|device| device.destroy_device(None));
        state
            .surface_loader
            .as_ref()
            .map(|surface_loader| surface_loader.destroy_surface(state.surface, None));
        state
            .instance
            .as_ref()
            .map(|instance| instance.destroy_instance(None));
    }
}
