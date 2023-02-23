mod ffi;
mod renderer;

pub use ffi::{
    AppMessage, Fullscreen, MessageWindowFn, SafeCString, WindowInstance, WindowMessage,
};
pub use renderer::Renderer;

pub struct WindowMessenger {
    messenger: MessageWindowFn,
}

impl WindowMessenger {
    pub unsafe fn from_raw(messenger: MessageWindowFn) -> Self {
        Self { messenger }
    }

    pub fn send(&self, w: ffi::WindowInstance, msg: &AppMessage) {
        unsafe { (self.messenger)(w, msg as *const AppMessage) };
    }
}

pub unsafe fn run_app(
    w: ffi::WindowInstance,
    message_window: ffi::MessageWindowFn,
    entry_fn: impl FnOnce(ffi::WindowInstance, WindowMessenger) + std::panic::UnwindSafe,
) {
    std::panic::catch_unwind(|| {
        entry_fn(w, WindowMessenger::from_raw(message_window));
    })
    .unwrap_or_else(|e| {
        log::error!("App panicked: {e:?}");
        std::process::abort();
    });
}

#[macro_export]
macro_rules! app_entry {
    ($entry_fn:ident) => {
        #[no_mangle]
        pub unsafe extern "C" fn faisca_run_app(
            w: $crate::WindowInstance,
            message_window: $crate::MessageWindowFn,
        ) {
            $crate::run_app(w, message_window, $entry_fn);
        }
    };
}

#[no_mangle]
pub unsafe extern "C" fn faisca_message_app(
    w: ffi::WindowInstance,
    msg: *const WindowMessage,
) -> u32 {
    std::panic::catch_unwind(|| {
        match *msg {
            WindowMessage::VulkanRequiredInstanceExtensions { names, count } => {
                let names = std::slice::from_raw_parts(names, count).to_vec();
            }
        }
        0
    })
    .unwrap_or_else(|e| {
        log::error!("App panicked: {e:?}");
        std::process::abort()
    })
}
