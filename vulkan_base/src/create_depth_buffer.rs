use ash::vk;

pub fn create_depth_buffer(
    device: &ash::Device,
    surface_extent: &vk::Extent2D,
    depth_format: vk::Format,
    allocator: &mut gpu_allocator::vulkan::Allocator,
) -> Result<vulkan_utils::MemImage, String> {
    // image
    log::info!("creating depth buffer image");

    let extent = vk::Extent3D {
        width: surface_extent.width,
        height: surface_extent.height,
        depth: 1,
    };

    let image_sg = {
        let image_create_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(depth_format)
            .extent(extent)
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .build();

        let image = unsafe {
            device
                .create_image(&image_create_info, None)
                .map_err(|_| format!("failed to create depth buffer image"))?
        };

        scopeguard::guard(image, |image| {
            log::warn!("depth buffer image scopeguard");
            unsafe {
                device.destroy_image(image, None);
            }
        })
    };

    log::info!("depth buffer image created");

    // allocation
    log::info!("allocating depth buffer image memory");

    let allocation_sg = {
        let memory_requirements = unsafe { device.get_image_memory_requirements(*image_sg) };

        let allocation_create_desc = gpu_allocator::vulkan::AllocationCreateDesc {
            name: "depth buffer image",
            requirements: memory_requirements,
            location: gpu_allocator::MemoryLocation::GpuOnly,
            linear: false,
        };

        let allocation = allocator
            .allocate(&allocation_create_desc)
            .map_err(|_| format!("failed to allocate depth buffer image memory"))?;

        scopeguard::guard(allocation, |allocation| {
            log::warn!("depth buffer image allocation scopeguard");
            let _ = allocator.free(allocation);
        })
    };

    log::info!("depth buffer image memory allocated");

    // binding
    log::info!("binding depth buffer image memory");

    unsafe {
        device
            .bind_image_memory(*image_sg, allocation_sg.memory(), allocation_sg.offset())
            .map_err(|_| format!("failed to bind depth buffer image memory"))?
    };

    log::info!("depth buffer image memory bound");

    // view
    log::info!("creating depth buffer image view");

    let image_view_sg = {
        let view_create_info = vk::ImageViewCreateInfo::builder()
            .image(*image_sg)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(depth_format)
            .components(vk::ComponentMapping {
                r: vk::ComponentSwizzle::R,
                g: vk::ComponentSwizzle::G,
                b: vk::ComponentSwizzle::B,
                a: vk::ComponentSwizzle::A,
            })
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::DEPTH,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            })
            .build();

        let view = unsafe {
            device
                .create_image_view(&view_create_info, None)
                .map_err(|_| format!("failed to create depth buffer image view"))?
        };

        scopeguard::guard(view, |view| {
            log::warn!("depth buffer image view scopeguard");
            unsafe {
                device.destroy_image_view(view, None);
            }
        })
    };

    log::info!("depth buffer image view created");

    Ok(vulkan_utils::MemImage {
        image: scopeguard::ScopeGuard::into_inner(image_sg),
        view: scopeguard::ScopeGuard::into_inner(image_view_sg),
        extent,
        allocation: scopeguard::ScopeGuard::into_inner(allocation_sg),
    })
}
