mod ffi;
mod renderer;
mod util;

use std::sync::RwLock;

pub use ffi::{
    AppMessage, Fullscreen, MessageWindowFn, SafeCString, WindowInstance, WindowMessage,
};
pub use renderer::Renderer;

#[cfg(debug_assertions)]
pub const DEBUG_ENABLED: bool = true;
#[cfg(not(debug_assertions))]
pub const DEBUG_ENABLED: bool = false;

const VK_LAYER_KHRONOS_VALIDATION: &'static [u8] = b"VK_LAYER_KHRONOS_validation\0";
const VK_VALIDATION_LAYERS: [*const i8; 1] =
    [VK_LAYER_KHRONOS_VALIDATION.as_ptr() as *const i8];

const VK_EXT_DEBUG_UTILS_EXTENSION_NAME: &'static [u8] = b"VK_EXT_debug_utils\0";

const VK_KHR_SWAPCHAIN_EXTENSION_NAME: &'static [u8] = b"VK_KHR_swapchain\0";
const VK_REQUIRED_DEVICE_EXTENSIONS: [*const i8; 1] =
    [VK_KHR_SWAPCHAIN_EXTENSION_NAME.as_ptr() as *const i8];

#[derive(Clone, Copy)]
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

static VK_INSTANCE_EXTENSIONS_VEC: RwLock<Vec<usize>> = RwLock::new(Vec::new());

pub unsafe fn run_app(
    w: ffi::WindowInstance,
    message_window: ffi::MessageWindowFn,
    entry_fn: impl FnOnce(ffi::WindowInstance, WindowMessenger) + std::panic::UnwindSafe,
) {
    entry_fn(w, WindowMessenger::from_raw(message_window));
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
    _w: ffi::WindowInstance,
    msg: *const WindowMessage,
) -> u32 {
    match *msg {
        WindowMessage::VulkanRequiredInstanceExtensions { names, count } => {
            let names = names as *const usize;
            let names = std::slice::from_raw_parts(names, count).to_vec();
            let mut write_guard = VK_INSTANCE_EXTENSIONS_VEC.write().unwrap();
            *write_guard = names;
        }
        WindowMessage::ResponseNotify { binding_address } => {
            unsafe { &*binding_address }.notify();
        }
    }
    0
}
