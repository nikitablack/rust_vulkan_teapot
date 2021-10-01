pub fn create_entry() -> Result<ash::Entry, String> {
    log::info!("creating entry");

    let entry = unsafe { ash::Entry::new().map_err(|_| String::from("failed to create Entry"))? };

    log::info!("entry created");

    Ok(entry)
}
