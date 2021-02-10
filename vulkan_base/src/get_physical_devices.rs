use ash::extensions::khr;
use ash::version::InstanceV1_0;
use ash::vk;

fn check_required_device_extensions(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    required_extensions: &Vec<&std::ffi::CStr>,
) -> Result<(), String> {
    let supported_device_extensions =
        match unsafe { instance.enumerate_device_extension_properties(physical_device) } {
            Ok(props) => props,
            Err(_) => {
                return Err(String::from(
                    "failed to enumerate instance extension properies",
                ))
            }
        };

    let mut supported_device_extensions_set = std::collections::HashSet::new();
    for vk::ExtensionProperties { extension_name, .. } in &supported_device_extensions {
        supported_device_extensions_set
            .insert(unsafe { std::ffi::CStr::from_ptr(extension_name.as_ptr()) });
    }

    for extension_name in required_extensions {
        if !supported_device_extensions_set.contains(extension_name) {
            return Err(format!(
                "device extension {:?} is not supported",
                extension_name
            ));
        }
    }

    Ok(())
}

fn check_device_suitability(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    required_extensions: &Vec<&std::ffi::CStr>,
    properties: &vk::PhysicalDeviceProperties,
) -> Result<(), String> {
    if vk::version_major(properties.api_version) < 1
        && vk::version_minor(properties.api_version) < 1
    {
        return Err(String::from(
            "the device does not support API version 1.1.0",
        ));
    }

    if properties.device_type != vk::PhysicalDeviceType::DISCRETE_GPU {
        return Err(String::from("the device is not a discrete GPU"));
    }

    let features = unsafe { instance.get_physical_device_features(physical_device) };

    // TODO pass as parameter
    if features.tessellation_shader == 0 {
        return Err(String::from(
            "the device does not support tesselation shader",
        ));
    }

    if features.fill_mode_non_solid == 0 {
        return Err(String::from(
            "the device does not support fill mode non solid",
        ));
    }

    check_required_device_extensions(instance, physical_device, required_extensions)?;

    Ok(())
}

fn get_surface_format(
    physical_device: vk::PhysicalDevice,
    surface_loader: &khr::Surface,
    surface: vk::SurfaceKHR,
) -> Result<vk::SurfaceFormatKHR, String> {
    let formats = match unsafe {
        surface_loader.get_physical_device_surface_formats(physical_device, surface)
    } {
        Ok(formats) => formats,
        Err(_) => {
            return Err(String::from(
                "failed to get physical device surface formats",
            ));
        }
    };

    if formats.is_empty() {
        return Err(String::from(
            "failed to get physical device surface formats",
        ));
    }

    if formats.len() == 1 && formats[0].format == vk::Format::UNDEFINED {
        return Ok(vk::SurfaceFormatKHR {
            format: vk::Format::B8G8R8A8_UNORM,
            color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
        });
    }

    for f in &formats {
        if f.format == vk::Format::B8G8R8A8_UNORM
            && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        {
            return Ok(vk::SurfaceFormatKHR {
                format: vk::Format::B8G8R8A8_UNORM,
                color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
            });
        }
    }

    Ok(formats[0])
}

fn get_present_mode(
    physical_device: vk::PhysicalDevice,
    surface_loader: &khr::Surface,
    surface: vk::SurfaceKHR,
) -> Result<vk::PresentModeKHR, String> {
    let modes = match unsafe {
        surface_loader.get_physical_device_surface_present_modes(physical_device, surface)
    } {
        Ok(formats) => formats,
        Err(_) => {
            return Err(String::from(
                "failed to get physical device surface present modes",
            ));
        }
    };

    if modes.is_empty() {
        return Err(String::from(
            "failed to get physical device surface present modes",
        ));
    }

    if modes.contains(&vk::PresentModeKHR::MAILBOX) {
        return Ok(vk::PresentModeKHR::MAILBOX);
    }

    if modes.contains(&vk::PresentModeKHR::IMMEDIATE) {
        return Ok(vk::PresentModeKHR::IMMEDIATE);
    }

    Ok(vk::PresentModeKHR::FIFO)
}

fn get_queue_family(
    physical_device: vk::PhysicalDevice,
    instance_loader: &ash::Instance,
    surface_loader: &khr::Surface,
    surface: vk::SurfaceKHR,
) -> Result<u32, String> {
    let props =
        unsafe { instance_loader.get_physical_device_queue_family_properties(physical_device) };

    for (ind, p) in props.iter().enumerate() {
        if p.queue_count > 0 && p.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
            let present_supported = match unsafe {
                surface_loader.get_physical_device_surface_support(
                    physical_device,
                    ind as u32,
                    surface,
                )
            } {
                Ok(result) => result,
                Err(_) => {
                    return Err(String::from(
                        "failed to get physical device surface_support",
                    ))
                }
            };

            if present_supported {
                return Ok(ind as u32);
            }
        }
    }

    Err(String::from(
        "failed to find graphics queue with present support",
    ))
}

fn get_depth_format(
    physical_device: vk::PhysicalDevice,
    instance_loader: &ash::Instance,
) -> Result<vk::Format, String> {
    let format_candidates = [
        vk::Format::D24_UNORM_S8_UINT,
        vk::Format::D32_SFLOAT_S8_UINT,
        vk::Format::D16_UNORM_S8_UINT,
    ];

    for &format in &format_candidates {
        let props = unsafe {
            instance_loader.get_physical_device_format_properties(physical_device, format)
        };

        if props
            .optimal_tiling_features
            .contains(vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT)
        {
            return Ok(format);
        }
    }

    Err(String::from("failed to find depth format"))
}

fn get_physical_device_data(
    physical_device: vk::PhysicalDevice,
    instance: &ash::Instance,
    surface_loader: &khr::Surface,
    surface: vk::SurfaceKHR,
) -> Result<crate::PhysicalDeviceData, String> {
    let mut data = crate::PhysicalDeviceData::default();

    data.physical_device = physical_device;
    data.surface_format = get_surface_format(physical_device, surface_loader, surface)?;
    data.present_mode = get_present_mode(physical_device, surface_loader, surface)?;
    data.queue_family = get_queue_family(physical_device, instance, surface_loader, surface)?;
    data.depth_format = get_depth_format(physical_device, instance)?;

    Ok(data)
}

pub fn get_physical_devices(
    vulkan_data: &mut crate::VulkanBaseData,
    device_extensions: &Vec<&'static std::ffi::CStr>,
) -> crate::VulkanInitResult {
    debug_assert!(vulkan_data.physical_devices.is_empty());

    log::info!("required device extensions: {:?}", device_extensions);

    let devices = match unsafe { vulkan_data.get_instance_ref().enumerate_physical_devices() } {
        Ok(devices) => devices,
        Err(_) => return Err(String::from("failed to enumerate physical devices")),
    };

    for device in devices {
        let properties = unsafe {
            vulkan_data
                .get_instance_ref()
                .get_physical_device_properties(device)
        };

        if let Err(msg) = check_device_suitability(
            vulkan_data.get_instance_ref(),
            device,
            device_extensions,
            &properties,
        ) {
            log::warn!("{}", msg);
            continue;
        }

        let device_name = unsafe { std::ffi::CStr::from_ptr(properties.device_name.as_ptr()) };

        let data = match get_physical_device_data(
            device,
            vulkan_data.get_instance_ref(),
            vulkan_data.get_surface_loader_ref(),
            vulkan_data.surface,
        ) {
            Ok(mut data) => {
                data.properties = properties;
                data
            }
            Err(msg) => {
                log::warn!(
                    "failed to get device {:?}. The reason: {}",
                    device_name,
                    msg
                );
                continue;
            }
        };

        log::info!("physical device {}", vulkan_data.physical_devices.len());
        log::info!("\tname: {:?}", device_name);
        log::info!(
            "\tsupported api version: {}.{}.{}",
            vk::version_major(properties.api_version),
            vk::version_minor(properties.api_version),
            vk::version_patch(properties.api_version)
        );
        log::info!("\tdriver version: {}", properties.driver_version);
        log::info!("\tsurface format: {:?}", data.surface_format);
        log::info!("\tpresent mode: {:?}", data.present_mode);
        log::info!("\tdepth format: {:?}", data.depth_format);
        log::info!("\tqueue family: {}", data.queue_family);

        vulkan_data.physical_devices.push(data);
    }

    if vulkan_data.physical_devices.is_empty() {
        return Err(String::from("failed to find suitable device"));
    }

    Ok(())
}
