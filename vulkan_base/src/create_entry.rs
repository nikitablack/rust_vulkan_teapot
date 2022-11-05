pub fn create_entry() -> ash::Entry {
    log::info!("creating entry");

    let entry = ash::Entry::linked();

    log::info!("entry created");

    entry
}
