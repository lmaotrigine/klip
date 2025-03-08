#[cfg(not(target_vendor = "uwp"))]
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;
#[cfg(not(target_vendor = "uwp"))]
use windows_sys::Win32::{
    Foundation::{MAX_PATH, S_OK},
    UI::Shell::{SHGetFolderPathW, CSIDL_PROFILE},
};

#[cfg(not(target_vendor = "uwp"))]
extern "C" {
    fn wcslen(buf: *const u16) -> usize;
}

#[cfg_attr(target_vendor = "uwp", allow(clippy::missing_const_for_fn))]
fn home_dir_crt() -> Option<PathBuf> {
    #[cfg(target_vendor = "uwp")]
    {
        None
    }
    #[cfg(not(target_vendor = "uwp"))]
    {
        let mut path = Vec::with_capacity(MAX_PATH as _);
        #[allow(clippy::cast_possible_wrap)] // 40 as i32 doesn't wrap.
        unsafe {
            match SHGetFolderPathW(
                core::ptr::null_mut(),
                CSIDL_PROFILE as _,
                core::ptr::null_mut(),
                0,
                path.as_mut_ptr(),
            ) {
                S_OK => {
                    let len = wcslen(path.as_ptr());
                    path.set_len(len);
                    let s = std::ffi::OsString::from_wide(&path);
                    Some(PathBuf::from(s))
                }
                _ => None,
            }
        }
    }
}

#[must_use]
pub fn home_dir() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .or_else(home_dir_crt)
}
