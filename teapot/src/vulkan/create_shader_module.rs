use crate::vulkan;
use ash::vk;
use std::io::Read;

pub fn create_shader_module(
    device: &ash::Device,
    debug_utils: &Option<ash::extensions::ext::DebugUtils>,
    path: &std::path::Path,
    object_name: &str,
) -> Result<vk::ShaderModule, String> {
    let mut file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return Err(format!("failed to open file {:?}", path)),
    };

    let mut spirv_u8 = Vec::new();
    if let Err(_) = file.read_to_end(&mut spirv_u8) {
        return Err(format!("failed to read file {:?}", path));
    }

    let spirv_u32 = match ash::util::read_spv(&mut std::io::Cursor::new(&spirv_u8)) {
        Ok(buf) => buf,
        Err(_) => return Err(format!("failed to read spirv {:?}", path)),
    };

    let create_info = vk::ShaderModuleCreateInfo::builder()
        .code(&spirv_u32)
        .build();

    let shader_module = match unsafe { device.create_shader_module(&create_info, None) } {
        Ok(module) => module,
        Err(_) => return Err(format!("failed to create shader module {:?}", path)),
    };

    vulkan::set_debug_utils_object_name(debug_utils, device.handle(), shader_module, object_name);

    log::info!("{} created", object_name);

    Ok(shader_module)
}
