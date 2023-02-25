use faisca::{AppMessage, Renderer, SafeCString, WindowInstance, WindowMessenger};

fn entry(w: WindowInstance, messenger: WindowMessenger) {
    env_logger::init();
    log::info!("Log enabled");
    messenger.send(
        w,
        &AppMessage::SetWindowTitle(SafeCString::allocate_from_str("Mamamia").unwrap()),
    );

    let _renderer = Renderer::new(w, messenger).unwrap_or_else(|e| {
        log::error!("Failed to create renderer: {e}");
        std::process::abort();
    });
}

faisca::app_entry!(entry);
