use std::{ffi::{CString, CStr, NulError}, ops::Deref};

#[repr(transparent)]
pub struct SafeCString(*const i8);

impl SafeCString {
    pub fn allocate_from_str(text: impl AsRef<str>) -> Result<SafeCString, NulError> {
        let string = CString::new(text.as_ref())?;
        let pointer = string.as_ptr();
        std::mem::forget(string);
        Ok(SafeCString(pointer))
    }
}
impl Deref for SafeCString {
    type Target = CStr;

    fn deref(&self) -> &Self::Target {
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
pub enum RendererMessage {
    SetWindowSize {
        width: u32,
        height: u32,
    } = 1,
    SetFullscreen(Fullscreen),
    SetBorderless(bool),
    SetWindowTitle(SafeCString),
}

