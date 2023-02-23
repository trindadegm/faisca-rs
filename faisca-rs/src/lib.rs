mod ffi;
mod renderer;

pub use ffi::{WindowMessage, SafeCString, Fullscreen, AppMessage};
pub use renderer::Renderer;

pub struct WindowMessenger {
    messenger: unsafe extern "C" fn(*const WindowMessage) -> u32,
}

impl WindowMessenger {
    pub unsafe fn from_raw(messenger: unsafe extern "C" fn(*const WindowMessage) -> u32) -> Self {
        Self {
            messenger
        }
    }

    pub fn send(&self, msg: &WindowMessage) {
        unsafe { (self.messenger)(msg as *const WindowMessage) };
    }
}

#[macro_export]
macro_rules! app_entry {
    ($faisca_fn:ident) => {
        #[no_mangle]
        pub unsafe extern "C" fn faisca_run_app(message_window: extern "C" fn(*const WindowMessage) -> u32) {
            std::panic::catch_unwind(|| {
                $faisca_fn(WindowMessenger::from_raw(message_window));
            }).unwrap_or_else(|_| {
                std::process::abort();
            });
        }
    }
}

pub unsafe extern "C" fn faisca_message_app(msg: *const AppMessage) -> u32 {
    std::panic::catch_unwind(|| {
        match *msg {
            AppMessage::VulkanRequiredInstanceExtensions { names, count } => {
                let names = std::slice::from_raw_parts(names, count);
            }
        }
        0
    }).unwrap_or_else(|e| {
        log::error!("App panicked: {e:?}");
        std::process::abort()
    })
}