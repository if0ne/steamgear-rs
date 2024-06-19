use std::sync::Arc;

use steamgear_sys as sys;

use crate::core::callback::ClientCallbackContainer;

#[derive(Clone, Debug)]
pub struct SteamFriends {
    raw: *mut sys::ISteamFriends,
    container: Arc<ClientCallbackContainer>,
}

unsafe impl Send for SteamFriends {}
unsafe impl Sync for SteamFriends {}

impl SteamFriends {
    pub(crate) fn new(container: Arc<ClientCallbackContainer>) -> Self {
        unsafe {
            SteamFriends {
                raw: sys::SteamAPI_SteamFriends_v017(),
                container,
            }
        }
    }
}
