use crate::vulkan;
use ash::version::DeviceV1_0;
use ash::vk;

pub fn create_font_image_view(
    data: &mut vulkan::TitleScreenVulkanData,
    vulkan_base_data: &vulkan_base::VulkanBaseData,
) -> common::VulkanResult {
    let device_ref = vulkan_base_data.get_device_ref();

    let create_info = vk::ImageViewCreateInfo {
        image: data.font_mem_image.image,
        view_type: vk::ImageViewType::TYPE_2D,
        format: vk::Format::R8G8B8A8_UNORM,
        components: vk::ComponentMapping {
            r: vk::ComponentSwizzle::R,
            g: vk::ComponentSwizzle::G,
            b: vk::ComponentSwizzle::B,
            a: vk::ComponentSwizzle::A,
        },
        subresource_range: vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        },
        ..Default::default()
    };

    data.font_mem_image.view = match unsafe { device_ref.create_image_view(&create_info, None) } {
        Ok(v) => v,
        Err(_) => return Err(String::from("failed to create font image view")),
    };

    if let Some(debug_utils) = vulkan_base_data.debug_utils_loader.as_ref() {
        common::set_debug_utils_object_name(
            debug_utils,
            device_ref.handle(),
            data.font_mem_image.view,
            String::from("title screen font image view"),
        );
    }

    Ok(())
}
