use crate::core::{
    callback::{CallbackDispatcher, CallbackTyped},
    SteamClientInner,
};

use parking_lot::Mutex;
use steamgear_sys as sys;
use tokio::sync::broadcast::Sender;
use tokio_stream::wrappers::errors::BroadcastStreamRecvError;

#[derive(Clone, Debug)]
pub struct SteamShutdown;

impl CallbackTyped for SteamShutdown {
    const TYPE: u32 = sys::SteamShutdown_t_k_iCallback;

    type Raw = sys::SteamShutdown_t;

    fn from_raw(_raw: Self::Raw) -> Self {
        SteamShutdown
    }
}

#[derive(Debug, Default)]
pub struct SteamShutdownDispatcher {
    storage: Mutex<Option<Sender<SteamShutdown>>>,
}

impl CallbackDispatcher for SteamShutdownDispatcher {
    type Item = SteamShutdown;

    fn storage(&self) -> &Mutex<Option<Sender<Self::Item>>> {
        &self.storage
    }
}

impl SteamClientInner {
    pub fn on_steam_shutdown(
        &self,
    ) -> impl futures_core::Stream<Item = Result<SteamShutdown, BroadcastStreamRecvError>> {
        self.callback_container.steam_shutdown_callback.register()
    }
}
