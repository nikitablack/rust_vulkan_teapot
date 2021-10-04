use ash::vk;

pub fn create_swapchain_image_views(
    device: &ash::Device,
    swapchain_images: &Vec<vk::Image>,
    surface_format: &vk::SurfaceFormatKHR,
) -> Result<Vec<vk::ImageView>, String> {
    log::info!("creating swapchain images views");

    let mut swapchain_image_views = Vec::with_capacity(swapchain_images.len());

    for (i, &image) in swapchain_images.iter().enumerate() {
        let create_info = vk::ImageViewCreateInfo::builder()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(surface_format.format)
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
            })
            .build();

        let view = unsafe {
            device.create_image_view(&create_info, None).map_err(|_| {
                clear_image_views(device, &swapchain_image_views);
                format!("failed to create image view {}", i)
            })?
        };

        swapchain_image_views.push(view);
    }

    log::info!("swapchain images views created");

    Ok(swapchain_image_views)
}

fn clear_image_views(device: &ash::Device, image_views: &Vec<vk::ImageView>) {
    for &image_view in image_views {
        unsafe {
            device.destroy_image_view(image_view, None);
        };
    }
}
