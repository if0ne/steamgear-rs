use crate::core::{
    callback::{CallbackDispatcher, CallbackError, CallbackTyped},
    SteamClientInner,
};

use futures::{Future, TryFutureExt};
use steamgear_sys as sys;

#[derive(Clone, Debug)]
pub struct SteamShutdown;

impl CallbackTyped for SteamShutdown {
    const TYPE: u32 = sys::SteamShutdown_t_k_iCallback as u32;

    type Raw = sys::SteamShutdown_t;

    fn from_raw(_raw: Self::Raw) -> Self {
        SteamShutdown
    }
}

impl SteamClientInner {
    pub fn on_steam_shutdown(&self) -> impl Future<Output = Result<SteamShutdown, CallbackError>> {
        let receiver = self.callback_container.steam_shutdown_callback.register();
        receiver.map_err(|_| CallbackError::Canceled)
    }
}
