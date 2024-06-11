pub mod callbacks;

use steamgear_sys as sys;

#[derive(Clone, Debug)]
pub struct SteamApps(pub(crate) *mut sys::ISteamApps);

unsafe impl Send for SteamApps {}
unsafe impl Sync for SteamApps {}

impl SteamApps {
    pub(crate) fn new() -> Self {
        unsafe { SteamApps(sys::SteamAPI_SteamApps_v008()) }
    }
}
