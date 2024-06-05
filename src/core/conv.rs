use steamgear_sys as sys;

use super::enums::SteamApiInitError;

impl SteamApiInitError {
    pub(crate) fn from_raw(raw: sys::ESteamAPIInitResult, msg: sys::SteamErrMsg) -> Self {
        let msg = unsafe {
            let cstr = std::ffi::CStr::from_ptr(msg.as_ptr() as *const std::ffi::c_char);
            cstr.to_string_lossy().to_string()
        };

        match raw {
            sys::ESteamAPIInitResult::k_ESteamAPIInitResult_FailedGeneric => {
                Self::FailedGeneric(msg)
            }
            sys::ESteamAPIInitResult::k_ESteamAPIInitResult_NoSteamClient => {
                Self::NoSteamClient(msg)
            }
            sys::ESteamAPIInitResult::k_ESteamAPIInitResult_VersionMismatch => {
                Self::VersionMismatch(msg)
            }
            _ => unreachable!(),
        }
    }
}
