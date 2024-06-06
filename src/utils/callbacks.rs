use crate::core::callback::CallbackTyped;

use steamgear_sys as sys;

pub struct SteamShutdown;

impl CallbackTyped for SteamShutdown {
    const TYPE: u32 = sys::SteamShutdown_t_k_iCallback as u32;

    type Raw = sys::SteamShutdown_t;

    fn from_raw(_raw: Self::Raw) -> Self {
        SteamShutdown
    }
}