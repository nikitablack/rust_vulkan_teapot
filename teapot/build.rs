use std::fs;
use std::path::Path;

fn visit_dirs(
    dir: &Path,
    cb: &dyn Fn(&std::path::PathBuf, shaderc::ShaderKind),
) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                let path_buf = entry.path();
                if let Some(shader_kind) = get_shader_kind(&path_buf) {
                    cb(&path_buf, shader_kind);
                }
            }
        }
    }

    Ok(())
}

fn get_shader_kind(path_buf: &std::path::PathBuf) -> Option<shaderc::ShaderKind> {
    let extension = path_buf
        .extension()
        .expect("file has no extension")
        .to_str()
        .expect("extension cannot be converted to &str");

    match extension {
        "vert" => Some(shaderc::ShaderKind::Vertex),
        "frag" => Some(shaderc::ShaderKind::Fragment),
        "tese" => Some(shaderc::ShaderKind::TessEvaluation),
        "tesc" => Some(shaderc::ShaderKind::TessControl),
        _ => None,
    }
}

fn compile_shader(path_buf: &std::path::PathBuf, shader_kind: shaderc::ShaderKind) {
    let shader_str = fs::read_to_string(path_buf)
        .expect(&format!("failed to read shader {:?} to string", path_buf));

    let compiler = shaderc::Compiler::new().expect("failed to create shader compilier");

    println!("compiling shader {:?}", path_buf);

    let spv = compiler
        .compile_into_spirv(
            &shader_str,
            shader_kind,
            &path_buf.to_str().unwrap(),
            "main",
            None,
        )
        .expect(&format!("failed to compile shader {:?}", path_buf));

    let mut file_name = path_buf
        .file_name()
        .expect("shader file should have a name")
        .to_os_string();

    println!("cargo:rerun-if-changed={}", path_buf.display());

    file_name.push(".spv");

    let mut spv_path = path_buf
        .parent()
        .expect("failed to get shader file parent folder")
        .join("..")
        .join("..")
        .join("shaders");

    std::fs::create_dir_all(spv_path.clone()).expect(&format!(
        "failed to create directory for shader {:?}",
        path_buf
    ));

    spv_path.push(file_name);

    fs::write(spv_path, spv.as_binary_u8()).expect("failed to write shader binary");
}

fn main() -> Result<(), i32> {
    let shaders_dir = Path::new("shaders");

    if let Err(_) = visit_dirs(shaders_dir, &compile_shader) {
        return Err(1);
    }

    Ok(())
}
