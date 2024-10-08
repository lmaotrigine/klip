use windows_sys::Win32::System::LibraryLoader::{
    SetDefaultDllDirectories, LOAD_LIBRARY_SEARCH_SYSTEM32,
};

/// Windows preflight security mitigations.
///
/// This attempts to defend against malicious DLLs that *may* sit alongside klip
/// in the same directory.
#[allow(clippy::missing_panics_doc)] // should never realistically panic.
pub fn preflight() {
    // default to delay loading DLLs from the system directory.
    // for DLLs loaded at load time, this relies on the `/DELAYLOAD` linker
    // flag.
    // this is only necesary prior to Windows 10 RS1.
    let result = unsafe { SetDefaultDllDirectories(LOAD_LIBRARY_SEARCH_SYSTEM32) };
    // SetDefaultDllDirectories should never fail if given valid arguments.
    // but, just to be safe, bail if it did.
    assert_ne!(result, 0);
}
