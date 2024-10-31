use std::sync::atomic::AtomicU8;

pub(crate) mod call_result;
pub(crate) mod callback_dispatcher;
pub(crate) mod reactor;
pub(crate) mod utils;

pub(crate) static STEAM_INIT_STATUS: AtomicU8 = AtomicU8::new(SteamApiState::Stopped as u8);

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub(crate) enum SteamApiState {
    Stopped,
    Init,
    RunCallbacks,
}
