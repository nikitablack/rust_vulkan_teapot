use ash::extensions::khr;

pub fn create_surface_loader(entry: &ash::Entry, instance: &ash::Instance) -> khr::Surface {
    let surface_loader = khr::Surface::new(&entry, &instance);

    log::info!("surface loader created");

    surface_loader
}
