use steamgear_sys as sys;

use super::call_result::CallResultTyped;

#[derive(Clone, Copy, Debug)]
pub(crate) struct CallUtils {
    pub(crate) raw: *mut sys::ISteamUtils,
}

impl CallUtils {
    pub(crate) fn new() -> Self {
        unsafe {
            Self {
                raw: sys::SteamAPI_SteamUtils_v010(),
            }
        }
    }

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

    pub(crate) fn get_api_call_result<T: CallResultTyped>(
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
