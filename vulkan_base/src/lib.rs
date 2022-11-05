mod check_instance_version;
mod check_required_instance_extensions;
mod create_allocator;
mod create_debug_utils_loader;
mod create_depth_buffer;
mod create_entry;
mod create_instance;
mod create_logical_device;
mod create_surface;
mod create_surface_loader;
mod create_swapchain;
mod create_swapchain_image_views;
mod create_swapchain_loader;
mod get_depth_format;
mod get_physical_device;
mod get_physical_device_properties;
mod get_present_mode;
mod get_queue;
mod get_queue_family;
mod get_surface_capabilities;
mod get_surface_extent;
mod get_surface_format;
mod get_swapchain_images;

use ash::extensions::khr;
use ash::vk;
use scopeguard::{guard, ScopeGuard};

use check_instance_version::*;
use check_required_instance_extensions::*;
use create_allocator::*;
use create_debug_utils_loader::*;
use create_depth_buffer::*;
use create_entry::*;
use create_instance::*;
use create_logical_device::*;
use create_surface::*;
use create_surface_loader::*;
use create_swapchain::*;
use create_swapchain_image_views::*;
use create_swapchain_loader::*;
use get_depth_format::*;
use get_physical_device::*;
use get_physical_device_properties::*;
use get_present_mode::*;
use get_queue::*;
use get_queue_family::*;
use get_surface_capabilities::*;
use get_surface_extent::*;
use get_surface_format::*;
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
    pub depth_buffer_mem_image: vulkan_utils::MemImage,
}

impl VulkanBase {
    pub fn new<'a, 'b>(
        window: &winit::window::Window,
        required_instance_extensions: &Vec<&'a std::ffi::CStr>,
        required_device_extensions: &Vec<&'b std::ffi::CStr>,
    ) -> Result<Self, String> {
        let entry = create_entry();
        check_instance_version(&entry)?;
        check_required_instance_extensions(&entry, required_instance_extensions)?;

        let instance_sg = {
            let instance = create_instance(&entry, required_instance_extensions)?;
            guard(instance, |instance| {
                log::warn!("instance scopeguard");
                unsafe {
                    instance.destroy_instance(None);
                }
            })
        };

        let debug_utils_loader = create_debug_utils_loader(&entry, &instance_sg);
        let surface_loader = create_surface_loader(&entry, &instance_sg);

        let surface_sg = {
            let surface = create_surface(&entry, &instance_sg, window)?;
            guard(surface, |surface| {
                log::warn!("surface scopeguard");
                unsafe {
                    surface_loader.destroy_surface(surface, None);
                }
            })
        };

        let physical_device = get_physical_device(&instance_sg, &required_device_extensions)?;
        let physical_device_properties =
            get_physical_device_properties(&instance_sg, physical_device);
        let surface_format = get_surface_format(physical_device, &surface_loader, *surface_sg)?;
        let present_mode = get_present_mode(physical_device, &surface_loader, *surface_sg)?;
        let queue_family =
            get_queue_family(&instance_sg, physical_device, &surface_loader, *surface_sg)?;
        let depth_format = get_depth_format(&instance_sg, physical_device)?;

        let device_sg = {
            let device = create_logical_device(
                &instance_sg,
                physical_device,
                queue_family,
                &required_device_extensions,
            )?;
            guard(device, |device| {
                log::warn!("device scopeguard");
                unsafe {
                    device.destroy_device(None);
                }
            })
        };

        let queue = get_queue(&device_sg, queue_family);

        let mut allocator = create_allocator(&instance_sg, &device_sg, physical_device)?;

        let swapchain_loader = create_swapchain_loader(&instance_sg, &device_sg);

        let resize_data = resize_internal(
            window,
            &device_sg,
            &surface_loader,
            &swapchain_loader,
            physical_device,
            vk::SwapchainKHR::null(),
            *surface_sg,
            &surface_format,
            present_mode,
            &vec![],
            depth_format,
            &mut allocator,
            None,
        )?;

        let swapchain_sg = {
            guard(resize_data.swapchain, |swapchain| {
                log::warn!("swapchain scopeguard");
                unsafe {
                    swapchain_loader.destroy_swapchain(swapchain, None);
                }
            })
        };

        let swapchain_image_views_sg = {
            guard(resize_data.swapchain_image_views, |image_views| {
                log::warn!("swapchain image views scopeguard");
                for view in image_views {
                    unsafe {
                        device_sg.destroy_image_view(view, None);
                    }
                }
            })
        };

        Ok(VulkanBase {
            entry,
            instance: ScopeGuard::into_inner(instance_sg),
            surface: ScopeGuard::into_inner(surface_sg),
            surface_loader,
            debug_utils_loader,
            physical_device,
            physical_device_properties,
            surface_format,
            present_mode,
            depth_format,
            queue_family,
            queue,
            allocator,
            surface_capabilities: resize_data.surface_capabilities,
            surface_extent: resize_data.surface_extent,
            swapchain: ScopeGuard::into_inner(swapchain_sg),
            swapchain_images: resize_data.swapchain_images,
            swapchain_image_views: ScopeGuard::into_inner(swapchain_image_views_sg),
            swapchain_loader,
            device: ScopeGuard::into_inner(device_sg),
            depth_buffer_mem_image: resize_data.depth_buffer_mem_image,
        })
    }

    pub fn resize(&mut self, window: &winit::window::Window) -> Result<(), String> {
        let old_depth_buffer_mem_image = std::mem::take(&mut self.depth_buffer_mem_image);

        let resize_data = resize_internal(
            window,
            &self.device,
            &self.surface_loader,
            &self.swapchain_loader,
            self.physical_device,
            self.swapchain,
            self.surface,
            &self.surface_format,
            self.present_mode,
            &self.swapchain_image_views,
            self.depth_format,
            &mut self.allocator,
            Some(old_depth_buffer_mem_image),
        )?;

        self.surface_capabilities = resize_data.surface_capabilities;
        self.surface_extent = resize_data.surface_extent;
        self.swapchain = resize_data.swapchain;
        self.swapchain_images = resize_data.swapchain_images;
        self.swapchain_image_views = resize_data.swapchain_image_views;
        self.depth_buffer_mem_image = resize_data.depth_buffer_mem_image;

        Ok(())
    }

    pub fn clean(mut self) {
        log::info!("cleaning vulkan base");

        unsafe {
            self.device
                .destroy_image(self.depth_buffer_mem_image.image, None);
            self.device
                .destroy_image_view(self.depth_buffer_mem_image.view, None);
            let _ = self.allocator.free(self.depth_buffer_mem_image.allocation);
            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);
            for &image_view in &self.swapchain_image_views {
                self.device.destroy_image_view(image_view, None);
            }
            drop(self.allocator);
            self.device.destroy_device(None);
            self.surface_loader.destroy_surface(self.surface, None);
            self.instance.destroy_instance(None);
        }
    }
}

struct ResizeResult {
    surface_capabilities: vk::SurfaceCapabilitiesKHR,
    surface_extent: vk::Extent2D,
    swapchain: vk::SwapchainKHR,
    swapchain_images: Vec<vk::Image>,
    swapchain_image_views: Vec<vk::ImageView>,
    depth_buffer_mem_image: vulkan_utils::MemImage,
}

fn resize_internal(
    window: &winit::window::Window,
    device: &ash::Device,
    surface_loader: &ash::extensions::khr::Surface,
    swapchain_loader: &ash::extensions::khr::Swapchain,
    physical_device: vk::PhysicalDevice,
    old_swapchain: vk::SwapchainKHR,
    surface: vk::SurfaceKHR,
    surface_format: &vk::SurfaceFormatKHR,
    present_mode: vk::PresentModeKHR,
    old_swapchain_image_views: &Vec<vk::ImageView>,
    depth_format: vk::Format,
    allocator: &mut gpu_allocator::vulkan::Allocator,
    old_depth_buffer_mem_image: Option<vulkan_utils::MemImage>,
) -> Result<ResizeResult, String> {
    log::info!("resizing VulkanBase");

    unsafe {
        let _ = device.device_wait_idle();
    }

    let surface_capabilities = get_surface_capabilities(surface_loader, physical_device, surface)?;
    let surface_extent = get_surface_extent(window, &surface_capabilities);

    let swapchain_sg = {
        let swapchain = create_swapchain(
            old_swapchain,
            surface,
            &surface_capabilities,
            surface_format,
            surface_extent,
            present_mode,
            swapchain_loader,
        )?;
        guard(swapchain, |swapchain| {
            log::warn!("swapchain scopeguard");
            unsafe {
                swapchain_loader.destroy_swapchain(swapchain, None);
            }
        })
    };

    // no need to explicitly destroy images. They are destroyed when the swapchain is destroyed.
    let swapchain_images = get_swapchain_images(swapchain_loader, *swapchain_sg)?;

    if !old_swapchain_image_views.is_empty() {
        log::info!("destroying old swapchain image views");
        for &image_view in old_swapchain_image_views {
            unsafe {
                device.destroy_image_view(image_view, None);
            };
        }
    }

    let swapchain_image_view_sgs = {
        let swapchain_image_views =
            create_swapchain_image_views(device, &swapchain_images, surface_format)?;

        let mut sgs = Vec::with_capacity(swapchain_image_views.len());
        for (i, &image_view) in swapchain_image_views.iter().enumerate() {
            let sg = guard(image_view, move |image_view| {
                log::warn!("swapchain image view {} scopeguard", i);
                unsafe {
                    device.destroy_image_view(image_view, None);
                }
            });
            sgs.push(sg);
        }

        sgs
    };

    if let Some(mem_image) = old_depth_buffer_mem_image {
        log::info!("destroying old depth buffer");
        unsafe {
            device.destroy_image(mem_image.image, None);
            device.destroy_image_view(mem_image.view, None);
        }
        let _ = allocator.free(mem_image.allocation);
    }

    let depth_buffer_sg = {
        let depth_buffer_mem_image =
            create_depth_buffer(device, &surface_extent, depth_format, allocator)?;

        guard(depth_buffer_mem_image, |mem_image| {
            log::warn!("depth buffer mem image scopeguard");
            unsafe {
                device.destroy_image(mem_image.image, None);
                device.destroy_image_view(mem_image.view, None);
            }
            let _ = allocator.free(mem_image.allocation);
        })
    };

    Ok(ResizeResult {
        surface_capabilities,
        surface_extent,
        swapchain: ScopeGuard::into_inner(swapchain_sg),
        swapchain_images,
        swapchain_image_views: swapchain_image_view_sgs
            .into_iter()
            .map(|sg| ScopeGuard::into_inner(sg))
            .collect(),
        depth_buffer_mem_image: ScopeGuard::into_inner(depth_buffer_sg),
    })
}
