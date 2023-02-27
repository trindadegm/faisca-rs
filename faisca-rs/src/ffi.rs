use std::ffi::{CStr, CString, NulError};

#[repr(transparent)]
pub struct SafeCString(*const i8);

impl SafeCString {
    pub fn allocate_from_str(text: impl AsRef<str>) -> Result<SafeCString, NulError> {
        let string = CString::new(text.as_ref())?;
        let pointer = string.as_ptr();
        std::mem::forget(string);
        Ok(SafeCString(pointer))
    }

    pub fn as_cstr(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.0) }
    }
}
impl Drop for SafeCString {
    fn drop(&mut self) {
        unsafe {
            drop(CString::from_raw(self.0 as _));
        }
    }
}

#[repr(u8)]
pub enum Fullscreen {
    No = 0,
    Real = 1,
    BorderlessWindow = 2,
}

#[repr(C)]
pub struct ResponseBinding {
    out: *mut std::ffi::c_void,
    wait_flag: *mut std::ffi::c_void, //(std::sync::Mutex<bool>, std::sync::Condvar),
}

#[repr(C, u32)]
pub enum AppMessage {
    SetWindowSize {
        width: u32,
        height: u32,
    } = 1,
    SetFullscreen(Fullscreen),
    SetBorderless(bool),
    SetWindowTitle(SafeCString),
    /// `out_binding` is a vulkan surface handle binding
    CreateVulkanSurface {
        instance: u64,
        out_binding: *const ResponseBinding,
    },
    /// `out_binding` is a `Extent2D` binding
    QueryViewportExtents {
        out_binding: *const ResponseBinding,
    },
    SetMsgBackchannel {
        channel: *const std::ffi::c_void,
    },
}

#[repr(C, u32)]
pub enum WindowMessage {
    /// This message is only sent once at the beggining of the process execution.
    /// It contains the required instance extensions to be used by the Vulkan driver
    /// initialization.
    VulkanRequiredInstanceExtensions {
        /// A pointer representing a list of C strings, the strings are null terminated.
        names: *const *const i8,
        /// The number of C strings on the `names` list.
        count: usize,
    } = 1,
    ResponseNotify {
        binding_address: *const ResponseBinding,
    },
    WindowEvent {
        channel: *mut std::ffi::c_void,
        event: *const WindowEvent,
    },
}

#[derive(Clone, Copy)]
#[repr(C, u32)]
pub enum WindowEvent {
    Quit = 1,
    KeyDown { c: u32 },
}

#[repr(C)]
pub struct Extent2D {
    pub width: u32,
    pub height: u32,
}

impl ResponseBinding {
    pub unsafe fn new(out: *mut std::ffi::c_void) -> Self {
        let wait_flag = Box::new((std::sync::Mutex::new(false), std::sync::Condvar::new()));
        Self {
            wait_flag: Box::into_raw(wait_flag) as *mut std::ffi::c_void,
            out,
        }
    }

    pub fn wait_flag(&self) -> &(std::sync::Mutex<bool>, std::sync::Condvar) {
        unsafe { &*(self.wait_flag as *mut (std::sync::Mutex<bool>, std::sync::Condvar)) }
    }

    pub fn reset(&self) {
        let (ready_mutex, _) = self.wait_flag();
        let mut ready_guard = ready_mutex.lock().unwrap();
        *ready_guard = false;
    }

    pub fn wait(&self) {
        let (ready_mutex, ready_condvar) = self.wait_flag();
        let mut ready_guard = ready_mutex.lock().unwrap();
        while !*ready_guard {
            ready_guard = ready_condvar.wait(ready_guard).unwrap();
        }
    }

    pub fn notify(&self) {
        let (ready_mutex, ready_condvar) = self.wait_flag();
        let mut ready_guard = ready_mutex.lock().unwrap();
        *ready_guard = true;
        ready_condvar.notify_all();
    }
}
impl Drop for ResponseBinding {
    fn drop(&mut self) {
        let condvar = unsafe { Box::from_raw(self.wait_flag) };
        drop(condvar);
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct WindowInstance(usize);

impl WindowInstance {
    #[inline(always)]
    pub unsafe fn null() -> Self {
        Self(0)
    }
}

pub type MessageWindowFn = unsafe extern "C" fn(WindowInstance, *const AppMessage) -> u32;
