use faisca::{AppMessage, Renderer, SafeCString, WindowInstance, WindowMessenger, WindowEvent};

fn entry(w: WindowInstance, messenger: WindowMessenger) {
    env_logger::init();
    log::info!("Log enabled");
    messenger.send(
        w,
        &AppMessage::SetWindowTitle(SafeCString::allocate_from_str("Mamamia").unwrap()),
    );

    let mut renderer = Renderer::new(w, &messenger).unwrap_or_else(|e| {
        log::error!("Failed to create renderer: {e}");
        std::process::abort();
    });

    'app_loop: loop {
        if let Some((_msg_win, win_event)) = messenger.try_recv() {
            match win_event {
                WindowEvent::Quit => {
                    log::info!("Quitting application");
                    break 'app_loop;
                }
                _ => {}
            }
        }
        renderer.draw_frame().unwrap();
    }
}

faisca::app_entry!(entry);
