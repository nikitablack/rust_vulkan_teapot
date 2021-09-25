mod check_instance_version;
mod check_required_instance_extensions;
mod create_allocator;
mod create_debug_utils_loader;
mod create_entry;
mod create_instance;
mod create_logical_device;
mod create_surface;
mod create_surface_loader;
mod get_depth_format;
mod get_physical_device;
mod get_physical_device_properties;
mod get_present_mode;
mod get_queue;
mod get_queue_family;
mod get_surface_format;

use ash::extensions::khr;
use ash::vk;

use check_instance_version::*;
use check_required_instance_extensions::*;
use create_allocator::*;
use create_debug_utils_loader::*;
use create_entry::*;
use create_instance::*;
use create_logical_device::*;
use create_surface::*;
use create_surface_loader::*;
use get_depth_format::*;
use get_physical_device::*;
use get_physical_device_properties::*;
use get_present_mode::*;
use get_queue::*;
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

impl VulkanBase {
    pub fn new<'a, 'b>(
        window: &winit::window::Window,
        required_instance_extensions: &Vec<&'a std::ffi::CStr>,
        required_device_extensions: &Vec<&'b std::ffi::CStr>,
        enable_debug_utils: bool,
    ) -> Result<Self, String> {
        let entry = create_entry()?;
        check_instance_version(&entry)?;
        check_required_instance_extensions(&entry, required_instance_extensions)?;

        let instance = create_instance(&entry, required_instance_extensions)?;
        let instance_sg = scopeguard::guard(instance, |instance| {
            log::info!("something went wrong, destroying instance");
            unsafe {
                instance.destroy_instance(None);
            }
        });

        let debug_utils_loader =
            create_debug_utils_loader(enable_debug_utils, &entry, &instance_sg);
        let surface_loader = create_surface_loader(&entry, &instance_sg);

        let surface = create_surface(&entry, &instance_sg, window)?;
        let surface_sg = scopeguard::guard(surface, |surface| {
            log::info!("something went wrong, destroying surface");
            unsafe {
                surface_loader.destroy_surface(surface, None);
            }
        });

        let physical_device = get_physical_device(&instance_sg, &required_device_extensions)?;
        let physical_device_properties =
            get_physical_device_properties(&instance_sg, physical_device);
        let surface_format = get_surface_format(physical_device, &surface_loader, *surface_sg)?;
        let present_mode = get_present_mode(physical_device, &surface_loader, *surface_sg)?;
        let queue_family =
            get_queue_family(&instance_sg, physical_device, &surface_loader, *surface_sg)?;
        let depth_format = get_depth_format(&instance_sg, physical_device)?;

        let device = create_logical_device(
            &instance_sg,
            physical_device,
            queue_family,
            &required_device_extensions,
        )?;
        let device_sg = scopeguard::guard(device, |device| {
            log::info!("something went wrong, destroying device");
            unsafe {
                device.destroy_device(None);
            }
        });

        let queue = get_queue(&device_sg, queue_family);

        Ok(VulkanBase {
            entry,
            instance: scopeguard::ScopeGuard::into_inner(instance_sg),
            surface: scopeguard::ScopeGuard::into_inner(surface_sg),
            surface_loader,
            debug_utils_loader,
            physical_device,
            physical_device_properties,
            surface_format,
            present_mode,
            depth_format,
            queue_family,
            device: scopeguard::ScopeGuard::into_inner(device_sg),
            queue,
        })
    }

    pub fn clean(&mut self) {
        log::info!("cleaning vulkan base");

        self.allocator.destroy();

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
            allocator: None,
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
    state.entry =
        unsafe { Some(ash::Entry::new().map_err(|_| String::from("failed to create Entry"))?) };
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

    state.physical_device = get_physical_device(instance, &required_device_extensions)?;
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
        unsafe {
            self.device.destroy_device(None);
            self.surface_loader.destroy_surface(self.surface, None);
            self.instance.destroy_instance(None);
        }
    }
}
