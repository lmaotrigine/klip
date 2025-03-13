use std::path::PathBuf;
#[cfg(not(target_vendor = "uwp"))]
use std::{ffi::OsString, os::windows::ffi::OsStringExt};
#[cfg(not(target_vendor = "uwp"))]
use windows_sys::Win32::{
    Foundation::S_OK,
    System::Com::CoTaskMemFree,
    UI::Shell::{FOLDERID_Profile, SHGetKnownFolderPath, KF_FLAG_DONT_VERIFY},
};

#[cfg(not(target_vendor = "uwp"))]
unsafe extern "C" {
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
        let mut path = core::ptr::null_mut();
        unsafe {
            if SHGetKnownFolderPath(
                &FOLDERID_Profile,
                KF_FLAG_DONT_VERIFY as _,
                core::ptr::null_mut(),
                &mut path,
            ) == S_OK
            {
                let slice = core::slice::from_raw_parts(path, wcslen(path));
                let s = OsString::from_wide(slice);
                CoTaskMemFree(path.cast());
                Some(PathBuf::from(s))
            } else {
                CoTaskMemFree(path.cast());
                None
            }
        }
    }
}

pub fn home_dir() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .or_else(home_dir_crt)
}
