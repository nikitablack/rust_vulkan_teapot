use crate::vulkan;
use ash::version::DeviceV1_0;
use ash::vk;

pub fn create_depth_buffer(
    vulkan_base: &vulkan_base::VulkanBase,
) -> Result<vulkan::MemImage, String> {
    let extent = vk::Extent3D {
        width: vulkan_base.surface_extent.width,
        height: vulkan_base.surface_extent.height,
        depth: 1,
    };

    let image_create_info = vk::ImageCreateInfo::builder()
        .image_type(vk::ImageType::TYPE_2D)
        .format(vulkan_base.depth_format)
        .extent(extent)
        .mip_levels(1)
        .array_layers(1)
        .samples(vk::SampleCountFlags::TYPE_1)
        .tiling(vk::ImageTiling::OPTIMAL)
        .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .build();

    let allocation_create_info = vk_mem::AllocationCreateInfo {
        usage: vk_mem::MemoryUsage::GpuOnly,
        ..Default::default()
    };

    let (image, allocation, allocation_info) = vulkan_base
        .allocator
        .create_image(&image_create_info, &allocation_create_info)
        .map_err(|_| String::from("failed to create depth buffer image"))?;

    vulkan::set_debug_utils_object_name(
        &vulkan_base.debug_utils_loader,
        vulkan_base.device.handle(),
        image,
        "depth buffer image",
    );

    vulkan::set_debug_utils_object_name(
        &vulkan_base.debug_utils_loader,
        vulkan_base.device.handle(),
        allocation_info.get_device_memory(),
        "depth buffer image device memory",
    );

    let view_create_info = vk::ImageViewCreateInfo::builder()
        .image(image)
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(vulkan_base.depth_format)
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
        vulkan_base
            .device
            .create_image_view(&view_create_info, None)
            .map_err(|_| {
                let _ = vulkan_base.allocator.destroy_image(image, &allocation);
                String::from("failed to create depth buffer image view")
            })?
    };

    vulkan::set_debug_utils_object_name(
        &vulkan_base.debug_utils_loader,
        vulkan_base.device.handle(),
        view,
        "depth buffer image view",
    );

    Ok(vulkan::MemImage {
        image: image,
        view: view,
        extent: extent,
        allocation: allocation,
    })
}
