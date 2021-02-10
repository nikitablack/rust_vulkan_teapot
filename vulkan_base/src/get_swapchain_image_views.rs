use ash::version::DeviceV1_0;
use ash::vk;

pub fn get_swapchain_image_views(
    vulkan_data: &mut crate::VulkanBaseData,
) -> crate::VulkanInitResult {
    debug_assert!(vulkan_data.swapchain_image_views.is_empty());

    let ref device_data = vulkan_data.physical_devices[vulkan_data.selected_physical_device_index];

    vulkan_data
        .swapchain_image_views
        .reserve(vulkan_data.swapchain_images.len());

    for (i, &image) in vulkan_data.swapchain_images.iter().enumerate() {
        let create_info = vk::ImageViewCreateInfo::builder()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(device_data.surface_format.format)
            .components(vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY,
            })
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });

        let view = match unsafe {
            vulkan_data
                .get_device_ref()
                .create_image_view(&create_info, None)
        } {
            Ok(view) => view,
            Err(_) => return Err(format!("failed to create image view {}", i)),
        };

        vulkan_data.swapchain_image_views.push(view);
    }

    Ok(())
}
