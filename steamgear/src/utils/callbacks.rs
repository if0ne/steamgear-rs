use crate::core::callback::{CallbackDispatcher, CallbackType, CallbackTyped};

use futures_core::Stream;
use steamgear_sys as sys;

use super::client::SteamUtilsClient;

#[derive(Clone, Debug)]
pub struct SteamShutdown;

impl CallbackTyped for SteamShutdown {
    const TYPE: CallbackType = CallbackType::SteamShutdown;

    type Raw = sys::SteamShutdown_t;
    type Mapped = Self;

    fn from_raw(_raw: Self::Raw) -> Self::Mapped {
        SteamShutdown
    }
}

impl SteamUtilsClient {
    pub fn on_steam_shutdown(&self) -> impl Stream<Item = SteamShutdown> {
        self.container.steam_shutdown_callback.register()
    }
}
