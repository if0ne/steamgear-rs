pub mod callback;
pub mod client;
pub mod conv;
pub mod enums;
pub mod server;
pub mod structs;

use enums::SteamApiInitError;
use std::sync::atomic::AtomicU8;

static STEAM_INIT_STATUS: AtomicU8 = AtomicU8::new(SteamApiState::Stopped as u8);

pub(crate) trait SteamApiInterface: Send + Sync {
    type InitArgs;

    fn init(args: Self::InitArgs) -> Result<Self, SteamApiInitError>
    where
        Self: Sized;
    fn shutdown(&self);
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
enum SteamApiState {
    Stopped,
    Init,
    RunCallbacks,
}

pub type AppId = u32;
