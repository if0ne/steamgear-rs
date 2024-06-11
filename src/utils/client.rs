use steamgear_sys as sys;

use crate::core::{callback::CallbackTyped, client::SteamApiClient};

#[derive(Clone, Debug)]
pub struct SteamUtilsClient(pub(crate) *mut sys::ISteamUtils);

unsafe impl Send for SteamUtilsClient {}
unsafe impl Sync for SteamUtilsClient {}

impl SteamUtilsClient {
    pub(crate) fn new() -> Self {
        unsafe { SteamUtilsClient(sys::SteamAPI_SteamUtils_v010()) }
    }
}

impl SteamApiClient {
    pub(crate) fn is_api_call_completed(&self, call: sys::SteamAPICall_t) -> Option<bool> {
        let mut failed = false;

        let result = unsafe {
            sys::SteamAPI_ISteamUtils_IsAPICallCompleted(self.steam_utils.0, call, &mut failed)
        };

        if !failed {
            Some(result)
        } else {
            None
        }
    }

    pub(crate) fn get_api_call_result<T: CallbackTyped>(
        &self,
        call: sys::SteamAPICall_t,
    ) -> Option<T> {
        unsafe {
            let mut raw_type: T::Raw = std::mem::zeroed();
            let mut failed = false;

            let result = {
                let raw_type = &mut raw_type;
                let raw_type = raw_type as *mut T::Raw;

                sys::SteamAPI_ISteamUtils_GetAPICallResult(
                    self.steam_utils.0,
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
