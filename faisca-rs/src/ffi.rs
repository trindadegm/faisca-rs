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

#[repr(C, u32)]
pub enum AppMessage {
    SetWindowSize { width: u32, height: u32 } = 1,
    SetFullscreen(Fullscreen),
    SetBorderless(bool),
    SetWindowTitle(SafeCString),
}

#[repr(C, u32)]
pub enum WindowMessage {
    VulkanRequiredInstanceExtensions {
        names: *const *const i8,
        count: usize,
    },
}

#[repr(transparent)]
pub struct WindowInstance(usize);

pub type MessageWindowFn = unsafe extern "C" fn(WindowInstance, *const AppMessage) -> u32;
