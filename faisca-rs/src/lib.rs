mod ffi;
mod renderer;
mod util;

use std::sync::RwLock;

pub use ffi::{
    AppMessage, Fullscreen, MessageWindowFn, SafeCString, WindowEvent, WindowInstance,
    WindowMessage,
};
pub use renderer::Renderer;

#[cfg(debug_assertions)]
pub const DEBUG_ENABLED: bool = true;
#[cfg(not(debug_assertions))]
pub const DEBUG_ENABLED: bool = false;

const VK_LAYER_KHRONOS_VALIDATION: &'static [u8] = b"VK_LAYER_KHRONOS_validation\0";
const VK_VALIDATION_LAYERS: [*const i8; 1] = [VK_LAYER_KHRONOS_VALIDATION.as_ptr() as *const i8];

const VK_EXT_DEBUG_UTILS_EXTENSION_NAME: &'static [u8] = b"VK_EXT_debug_utils\0";

const VK_KHR_SWAPCHAIN_EXTENSION_NAME: &'static [u8] = b"VK_KHR_swapchain\0";
const VK_REQUIRED_DEVICE_EXTENSIONS: [*const i8; 1] =
    [VK_KHR_SWAPCHAIN_EXTENSION_NAME.as_ptr() as *const i8];

type WChanMsg = (ffi::WindowInstance, WindowEvent);

pub struct WindowMessenger {
    messenger: MessageWindowFn,
    wchan_recv: std::sync::mpsc::Receiver<WChanMsg>,
}

impl WindowMessenger {
    pub unsafe fn from_raw(messenger: MessageWindowFn) -> Self {
        let (wchan_send, wchan_recv) = std::sync::mpsc::channel();
        let messenger = Self {
            messenger,
            wchan_recv,
        };

        let wchan_send_box = Box::new(wchan_send);
        messenger.send(
            ffi::WindowInstance::null(),
            &AppMessage::SetMsgBackchannel {
                channel: Box::into_raw(wchan_send_box) as *mut std::ffi::c_void,
            },
        );

        messenger
    }

    pub fn send(&self, w: ffi::WindowInstance, msg: &AppMessage) {
        unsafe { (self.messenger)(w, msg as *const AppMessage) };
    }

    pub fn try_recv(&self) -> Option<WChanMsg> {
        self.wchan_recv.try_recv().ok()
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

#[macro_export]
macro_rules! app_process {
    ($process_fn:ident) => {
        #[no_mangle]
        pub unsafe extern "C" fn faisca_process() -> i32 {
            $process_fn()
        }
    };
}

#[no_mangle]
pub unsafe extern "C" fn faisca_message_app(
    w: ffi::WindowInstance,
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
        WindowMessage::WindowEvent { channel, event } => {
            let channel =
                unsafe { Box::from_raw(channel as *mut std::sync::mpsc::Sender<WChanMsg>) };

            let event = unsafe { *event };
            if let WindowEvent::Quit = event {
                let _ = channel.send((w, event));
            } else {
                // If it is not a Quit event, we don't want to call the drop
                Box::leak(channel);
            }
        }
    }
    0
}
