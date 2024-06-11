use steamgear_sys::{
    ISteamUtils, SteamAPICall_t, SteamAPI_ISteamUtils_GetAPICallResult,
    SteamAPI_ISteamUtils_IsAPICallCompleted, SteamAPI_SteamUtils_v010,
};

use crate::core::callback::CallbackTyped;

#[derive(Clone, Debug)]
pub struct SteamUtilsClient(pub(crate) *mut ISteamUtils);

unsafe impl Send for SteamUtilsClient {}
unsafe impl Sync for SteamUtilsClient {}

impl SteamUtilsClient {
    pub(crate) fn new() -> Self {
        unsafe { SteamUtilsClient(SteamAPI_SteamUtils_v010()) }
    }

    pub(crate) fn is_api_call_completed(&self, call: SteamAPICall_t) -> Option<bool> {
        let mut failed = false;

        let result = unsafe { SteamAPI_ISteamUtils_IsAPICallCompleted(self.0, call, &mut failed) };

        if !failed {
            Some(result)
        } else {
            None
        }
    }

    pub(crate) fn get_api_call_result<T: CallbackTyped>(&self, call: SteamAPICall_t) -> Option<T> {
        unsafe {
            let mut raw_type: T::Raw = std::mem::zeroed();
            let mut failed = false;

            let result = {
                let raw_type = &mut raw_type;
                let raw_type = raw_type as *mut T::Raw;

                SteamAPI_ISteamUtils_GetAPICallResult(
                    self.0,
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
