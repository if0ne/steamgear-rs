use std::fmt::Display;

use steamgear_sys as sys;
use thiserror::Error;

use super::{callback::CallbackType, enums::SteamApiInitError};

impl SteamApiInitError {
    pub(crate) fn from_raw(raw: sys::ESteamAPIInitResult, msg: sys::SteamErrMsg) -> Self {
        let msg = unsafe {
            let cstr = std::ffi::CStr::from_ptr(msg.as_ptr() as *const std::ffi::c_char);
            cstr.to_string_lossy().to_string()
        };

        match raw {
            sys::ESteamAPIInitResult_k_ESteamAPIInitResult_FailedGeneric => {
                Self::FailedGeneric(msg)
            }
            sys::ESteamAPIInitResult_k_ESteamAPIInitResult_NoSteamClient => {
                Self::NoSteamClient(msg)
            }
            sys::ESteamAPIInitResult_k_ESteamAPIInitResult_VersionMismatch => {
                Self::VersionMismatch(msg)
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Copy, Debug, Error)]
pub(crate) struct UnknownCallback;

impl Display for UnknownCallback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unknown callback type")
    }
}

impl TryFrom<u32> for CallbackType {
    type Error = UnknownCallback;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value.try_into().unwrap() {
            sys::SteamShutdown_t_k_iCallback => Ok(CallbackType::SteamShutdown),
            sys::DlcInstalled_t_k_iCallback => Ok(CallbackType::DlcInstalled),
            sys::NewUrlLaunchParameters_t_k_iCallback => Ok(CallbackType::NewUrlLaunchParameters),
            _ => Err(UnknownCallback),
        }
    }
}
