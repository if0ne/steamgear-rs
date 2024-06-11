use crate::{
    core::callback::CallbackTyped,
    prelude::{callback::CallbackDispatcher, client::SteamApiClient, server::SteamApiServer},
};

use futures::Stream;
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

impl SteamApiClient {
    pub fn on_steam_shutdown(&self) -> impl Stream<Item = SteamShutdown> {
        self.callback_container.steam_shutdown_callback.register()
    }
}

impl SteamApiServer {
    pub fn on_steam_shutdown(&self) -> impl Stream<Item = SteamShutdown> {
        self.callback_container.steam_shutdown_callback.register()
    }
}
