use crate::core::{
    callback::{CallbackDispatcher, CallbackTyped},
    SteamClientInner,
};

use async_channel::{Receiver, Sender};
use parking_lot::Mutex;
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

#[derive(Debug, Default)]
pub struct SteamShutdownDispatcher {
    storage: Mutex<Option<(Sender<SteamShutdown>, Receiver<SteamShutdown>)>>,
}

impl CallbackDispatcher for SteamShutdownDispatcher {
    type Item = SteamShutdown;

    fn storage(&self) -> &Mutex<Option<(Sender<Self::Item>, Receiver<Self::Item>)>> {
        &self.storage
    }
}

impl SteamClientInner {
    pub fn on_steam_shutdown(&self) -> impl futures_core::Stream<Item = SteamShutdown> {
        self.callback_container.steam_shutdown_callback.register()
    }
}
