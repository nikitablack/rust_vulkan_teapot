use ash::extensions::khr;
use ash::vk;

#[derive(Default)]
pub struct PhysicalDeviceData {
    pub physical_device: vk::PhysicalDevice,
    pub properties: vk::PhysicalDeviceProperties,
    pub surface_format: vk::SurfaceFormatKHR,
    pub present_mode: vk::PresentModeKHR,
    pub depth_format: vk::Format,
    pub queue_family: u32,
    pub queue: vk::Queue,
}

#[derive(Default)]
pub struct VulkanBaseData {
    pub entry: Option<ash::Entry>,

    pub surface_loader: Option<khr::Surface>,
    pub swapchain_loader: Option<khr::Swapchain>,
    pub debug_utils_loader: Option<ash::extensions::ext::DebugUtils>,

    pub instance: Option<ash::Instance>,
    pub device: Option<ash::Device>,
    pub allocator: Option<vk_mem::Allocator>,

    pub physical_devices: Vec<PhysicalDeviceData>,
    pub selected_physical_device_index: usize,

    pub surface: vk::SurfaceKHR,
    pub surface_capabilities: vk::SurfaceCapabilitiesKHR,
    pub surface_extent: vk::Extent2D,
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_images: Vec<vk::Image>,
    pub swapchain_image_views: Vec<vk::ImageView>,
}

impl VulkanBaseData {
    pub fn get_entry_ref(&self) -> &ash::Entry {
        self.entry.as_ref().expect("entry shouldn't be empty")
    }

    pub fn get_instance_ref(&self) -> &ash::Instance {
        self.instance.as_ref().expect("instance shouldn't be empty")
    }

    pub fn get_surface_loader_ref(&self) -> &ash::extensions::khr::Surface {
        self.surface_loader
            .as_ref()
            .expect("surface loader shouldn't be empty")
    }

    pub fn get_device_ref(&self) -> &ash::Device {
        self.device.as_ref().expect("device shouldn't be empty")
    }

    pub fn get_swapchain_loader_ref(&self) -> &ash::extensions::khr::Swapchain {
        self.swapchain_loader
            .as_ref()
            .expect("swapchain loader shouldn't be empty")
    }

    pub fn get_allocator_ref(&self) -> &vk_mem::Allocator {
        self.allocator
            .as_ref()
            .expect("allocator shouldn't be empty")
    }

    pub fn get_allocator_mut(&mut self) -> &mut vk_mem::Allocator {
        self.allocator
            .as_mut()
            .expect("allocator shouldn't be empty")
    }
}

pub type VulkanInitResult = Result<(), String>;
