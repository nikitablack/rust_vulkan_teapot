use crate::vulkan;
use ash::vk;

pub fn create_font_image(
    data: &mut vulkan::TitleScreenVulkanData,
    vulkan_base_data: &vulkan_base::VulkanBaseData,
    image_size: &[u32; 2],
) -> common::VulkanResult {
    let image_create_info = vk::ImageCreateInfo::builder()
        .image_type(vk::ImageType::TYPE_2D)
        .format(vk::Format::R8G8B8A8_UNORM)
        .extent(vk::Extent3D {
            width: image_size[0],
            height: image_size[1],
            depth: 1,
        })
        .mip_levels(1)
        .array_layers(1)
        .samples(vk::SampleCountFlags::TYPE_1)
        .tiling(vk::ImageTiling::OPTIMAL)
        .usage(vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .initial_layout(vk::ImageLayout::UNDEFINED);

    let allocation_create_info = vk_mem::AllocationCreateInfo {
        usage: vk_mem::MemoryUsage::GpuOnly,
        ..Default::default()
    };

    data.font_mem_image = match vulkan_base_data
        .get_allocator_ref()
        .create_image(&image_create_info, &allocation_create_info)
    {
        Ok((img, allocation, allocation_info)) => common::MemImage {
            image: img,
            view: vk::ImageView::null(),
            allocation,
            allocation_info: Some(allocation_info),
            extent: image_create_info.extent,
        },
        Err(_) => return Err(format!("failed to allocate font image")),
    };

    let device = vulkan_base_data.get_device_ref();

    if let Some(debug_utils) = vulkan_base_data.debug_utils_loader.as_ref() {
        common::set_debug_utils_object_name(
            debug_utils,
            device.handle(),
            data.font_mem_image.image,
            String::from("title screen font image"),
        );

        common::set_debug_utils_object_name(
            debug_utils,
            device.handle(),
            data.font_mem_image
                .get_allocation_info_ref()
                .get_device_memory(),
            String::from("title screen font image memory"),
        );
    }

    Ok(())
}
