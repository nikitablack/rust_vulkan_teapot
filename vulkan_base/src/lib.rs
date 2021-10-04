mod check_instance_version;
mod check_required_instance_extensions;
mod create_allocator;
mod create_debug_utils_loader;
mod create_entry;
mod create_instance;
mod create_logical_device;
mod create_surface;
mod create_surface_loader;
mod create_swapchain;
mod get_depth_format;
mod get_physical_device;
mod get_physical_device_properties;
mod get_present_mode;
mod get_queue;
mod get_queue_family;
mod get_surface_capabilities;
mod get_surface_extent;
mod get_surface_format;
mod get_swapchain_image_views;
mod get_swapchain_images;

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
use create_swapchain::*;
use get_depth_format::*;
use get_physical_device::*;
use get_physical_device_properties::*;
use get_present_mode::*;
use get_queue::*;
use get_queue_family::*;
use get_surface_capabilities::*;
use get_surface_extent::*;
use get_surface_format::*;
use get_swapchain_image_views::*;
use get_swapchain_images::*;

pub struct VulkanBase {
    pub entry: ash::Entry,
    pub instance: ash::Instance,
    pub surface_loader: khr::Surface,
    pub swapchain_loader: khr::Swapchain,
    pub debug_utils_loader: ash::extensions::ext::DebugUtils,
    pub surface: vk::SurfaceKHR,
    pub physical_device: vk::PhysicalDevice,
    pub physical_device_properties: vk::PhysicalDeviceProperties,
    pub surface_format: vk::SurfaceFormatKHR,
    pub present_mode: vk::PresentModeKHR,
    pub depth_format: vk::Format,
    pub queue_family: u32,
    pub device: ash::Device,
    pub queue: vk::Queue,
    pub allocator: gpu_allocator::vulkan::Allocator,
    pub surface_capabilities: vk::SurfaceCapabilitiesKHR,
    pub surface_extent: vk::Extent2D,
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_images: Vec<vk::Image>,
    pub swapchain_image_views: Vec<vk::ImageView>,
}

impl VulkanBase {
    pub fn new<'a, 'b>(
        window: &winit::window::Window,
        required_instance_extensions: &Vec<&'a std::ffi::CStr>,
        required_device_extensions: &Vec<&'b std::ffi::CStr>,
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

        let debug_utils_loader = create_debug_utils_loader(&entry, &instance_sg);
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

        let allocator = create_allocator(&instance_sg, &device_sg, physical_device)?;

        state.swapchain_loader = Some(ash::extensions::khr::Swapchain::new(instance, device));

        resize_internal(state, window)?;

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
            allocator,
        })
    }

    pub fn resize(&mut self, window: &winit::window::Window) -> Result<(), String> {
        unsafe {
            let _ = state.device.as_ref().unwrap().device_wait_idle();
        }

        state.surface_capabilities = get_surface_capabilities(
            state.surface_loader.as_ref().unwrap(),
            state.physical_device,
            state.surface,
        )?;
        state.surface_extent = get_surface_extent(window, &state.surface_capabilities);
        state.swapchain = create_swapchain(
            state.swapchain,
            state.surface,
            &state.surface_capabilities,
            &state.surface_format,
            state.surface_extent,
            state.present_mode,
            state.swapchain_loader.as_ref().unwrap(),
        )?;
        state.swapchain_images =
            get_swapchain_images(state.swapchain_loader.as_ref().unwrap(), state.swapchain)?;

        for &image_view in &state.swapchain_image_views {
            unsafe {
                state
                    .device
                    .as_ref()
                    .unwrap()
                    .destroy_image_view(image_view, None);
            };
        }

        state.swapchain_image_views = get_swapchain_image_views(
            state.device.as_ref().unwrap(),
            &state.swapchain_images,
            &state.surface_format,
        )?;

        Ok(())
    }

    pub fn clean(self) {
        log::info!("cleaning vulkan base");

        unsafe {
            swapchain_loader.destroy_swapchain(state.swapchain, None)
            for &image_view in &state.swapchain_image_views {
                device.destroy_image_view(image_view, None);
            }
            drop(self.allocator);
            self.device.destroy_device(None);
            self.surface_loader.destroy_surface(self.surface, None);
            self.instance.destroy_instance(None);
        }
    }
}
