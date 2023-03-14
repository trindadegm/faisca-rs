use faisca::{AppMessage, SafeCString, WindowEvent, WindowInstance, WindowMessenger, renderer::{Renderer, RendererError}};

fn entry(w: WindowInstance, messenger: WindowMessenger) {
    env_logger::init();
    log::info!("Log enabled");
    messenger.send(
        w,
        &AppMessage::SetWindowTitle(SafeCString::allocate_from_str("VkTut").unwrap()),
    );

    let mut win_extent_w = 800;
    let mut win_extent_h = 450;

    messenger.send(
        w,
        &AppMessage::SetWindowSize {
            width: win_extent_w,
            height: win_extent_h,
        },
    );

    messenger.send(
        w,
        &AppMessage::SetWindowResizable(true),
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
                WindowEvent::WindowResize { w, h } => {
                    log::debug!("Window resize event received: {w}, {h}");
                    win_extent_w = w;
                    win_extent_h = h;
                    renderer.window_resized(win_extent_w, win_extent_h)
                        .unwrap();
                }
            }
        }

        match renderer.draw_frame() {
            Ok(()) => (),
            Err(RendererError::FailedToDrawFrame(faisca::vk::Result::ERROR_OUT_OF_DATE_KHR)) => {
                log::warn!("OUT_OF_DATE_KHR error, skipping frame and resizing");
                renderer.window_resized(win_extent_w, win_extent_h)
                    .unwrap();
            },
            Err(e) => panic!("{e}"),
        }
    }
}

fn process() -> i32 {
    0
}

faisca::app_entry!(entry);
faisca::app_process!(process);
