use std::{ffi::CStr, sync::Arc};

use steamgear_sys as sys;

use crate::core::callback::{CallbackTyped, ClientCallbackContainer};

#[derive(Clone, Debug)]
pub struct SteamUtilsClient {
    pub(super) raw: *mut sys::ISteamUtils,
    pub(super) container: Arc<ClientCallbackContainer>,
}

unsafe impl Send for SteamUtilsClient {}
unsafe impl Sync for SteamUtilsClient {}

impl SteamUtilsClient {
    pub(crate) fn new(container: Arc<ClientCallbackContainer>) -> Self {
        unsafe {
            SteamUtilsClient {
                raw: sys::SteamAPI_SteamUtils_v010(),
                container,
            }
        }
    }
}

impl SteamUtilsClient {
    pub fn enable_debug_hook(&self) {
        unsafe {
            sys::SteamAPI_ISteamUtils_SetWarningMessageHook(
                self.raw,
                Some(_internal_warning_message_hook),
            )
        }
    }

    pub fn disable_debug_hook(&self) {
        unsafe { sys::SteamAPI_ISteamUtils_SetWarningMessageHook(self.raw, None) }
    }
}

impl SteamUtilsClient {
    pub(crate) fn is_api_call_completed(&self, call: sys::SteamAPICall_t) -> Option<bool> {
        let mut failed = false;

        let result =
            unsafe { sys::SteamAPI_ISteamUtils_IsAPICallCompleted(self.raw, call, &mut failed) };

        if !failed {
            Some(result)
        } else {
            None
        }
    }

    pub(crate) fn get_api_call_result<T: CallbackTyped>(
        &self,
        call: sys::SteamAPICall_t,
    ) -> Option<T::Mapped> {
        unsafe {
            let mut raw_type: T::Raw = std::mem::zeroed();
            let mut failed = false;

            let result = {
                let raw_type = &mut raw_type;
                let raw_type = raw_type as *mut T::Raw;

                sys::SteamAPI_ISteamUtils_GetAPICallResult(
                    self.raw,
                    call,
                    raw_type as *mut _,
                    std::mem::size_of::<T::Raw>() as i32,
                    T::TYPE as i32,
                    &mut failed,
                )
            };

            if !failed && result {
                Some(T::from_raw(raw_type))
            } else {
                None
            }
        }
    }
}

unsafe extern "C" fn _internal_warning_message_hook(
    severity: std::os::raw::c_int,
    message: *const std::os::raw::c_char,
) {
    let message = CStr::from_ptr(message);

    match severity {
        0 => {
            tracing::info!("{:?}", message);
        }
        1 => {
            tracing::warn!("{:?}", message);
        }
        _ => {
            unreachable!()
        }
    }
}
