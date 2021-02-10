use crate::vulkan;
use ash::version::DeviceV1_0;
use ash::vk;

pub fn create_sampler(
    data: &mut vulkan::TitleScreenVulkanData,
    vulkan_base_data: &vulkan_base::VulkanBaseData,
) -> common::VulkanResult {
    let device = vulkan_base_data.get_device_ref();

    let create_info = vk::SamplerCreateInfo {
        mag_filter: vk::Filter::LINEAR,
        min_filter: vk::Filter::LINEAR,
        mipmap_mode: vk::SamplerMipmapMode::LINEAR,
        address_mode_u: vk::SamplerAddressMode::REPEAT,
        address_mode_v: vk::SamplerAddressMode::REPEAT,
        address_mode_w: vk::SamplerAddressMode::REPEAT,
        mip_lod_bias: 0.0f32,
        anisotropy_enable: vk::FALSE,
        max_anisotropy: 0.0f32,
        compare_enable: vk::FALSE,
        compare_op: vk::CompareOp::NEVER,
        min_lod: 0.0f32,
        max_lod: 0.0f32,
        border_color: vk::BorderColor::FLOAT_OPAQUE_WHITE,
        unnormalized_coordinates: vk::FALSE,
        ..Default::default()
    };

    data.sampler = match unsafe { device.create_sampler(&create_info, None) } {
        Ok(s) => s,
        Err(_) => return Err(String::from("title screen sampler")),
    };

    if let Some(debug_utils) = vulkan_base_data.debug_utils_loader.as_ref() {
        common::set_debug_utils_object_name(
            debug_utils,
            device.handle(),
            data.sampler,
            String::from("sampler"),
        );
    }

    Ok(())
}
