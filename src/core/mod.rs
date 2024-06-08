pub mod callback;
pub mod conv;
pub mod enums;

use std::{ops::Deref, task::Waker};

use callback::{CallbackContainer, CallbackDispatcher, CallbackTyped};
use dashmap::DashMap;
use enums::SteamApiInitError;
use steamgear_sys as sys;

use crate::utils::{callbacks::SteamShutdown, SteamUtils};

#[derive(Debug)]
pub struct SteamClientInner {
    pipe: sys::HSteamPipe,
    call_results: DashMap<sys::SteamAPICall_t, Waker>,

    pub(crate) callback_container: CallbackContainer,
    pub(crate) steam_utils: SteamUtils,
}

impl SteamClientInner {
    pub fn restart_app_if_necessary(app_id: u32) -> bool {
        unsafe { sys::SteamAPI_RestartAppIfNecessary(app_id) }
    }

    pub fn run_callbacks(&self) {
        unsafe {
            sys::SteamAPI_ManualDispatch_RunFrame(self.pipe);
            let mut callback = std::mem::zeroed();
            while sys::SteamAPI_ManualDispatch_GetNextCallback(self.pipe, &mut callback) {
                if callback.m_iCallback as u32 == sys::SteamAPICallCompleted_t_k_iCallback as u32 {
                    let apicall =
                        &*(callback.m_pubParam as *const _ as *const sys::SteamAPICallCompleted_t);
                    let id = apicall.m_hAsyncCall;

                    if let Some(entry) = self.call_results.get(&id) {
                        entry.value().wake_by_ref();
                    }
                } else {
                    // TODO: Batched proceed
                    self.proceed_callback(callback);
                }
                sys::SteamAPI_ManualDispatch_FreeLastCallback(self.pipe);
            }
        }
    }
}

impl SteamClientInner {
    pub(crate) fn new(is_server: bool) -> Result<Self, SteamApiInitError> {
        unsafe {
            let pipe = if is_server {
                sys::SteamGameServer_GetHSteamPipe()
            } else {
                sys::SteamAPI_GetHSteamPipe()
            };

            Self::init_ex()?;

            sys::SteamAPI_ManualDispatch_Init();

            Ok(Self {
                pipe,
                call_results: Default::default(),
                steam_utils: SteamUtils::new(),
                callback_container: Default::default(),
            })
        }
    }

    pub(crate) fn register_call_result(&self, id: sys::SteamAPICall_t, waker: Waker) {
        self.call_results.insert(id, waker);
    }

    pub(crate) fn remove_call_result(&self, id: sys::SteamAPICall_t) {
        self.call_results.remove(&id);
    }

    /*pub(crate) fn register_call_back<T: CallbackTyped>(&self) -> impl Stream<Item = T> + Send {
        self.call_backs.register_call_back::<T>()
    }*/
}

impl SteamClientInner {
    fn init_ex() -> Result<(), SteamApiInitError> {
        let versions = Self::get_all_version();
        let versions: Vec<u8> = versions.into_iter().flatten().cloned().collect();
        let versions = versions.as_ptr() as *const ::std::os::raw::c_char;

        let mut err_msg: sys::SteamErrMsg = [0; 1024];

        let result = unsafe { sys::SteamInternal_SteamAPI_Init(versions, &mut err_msg) };

        match result {
            steamgear_sys::ESteamAPIInitResult_k_ESteamAPIInitResult_OK => Ok(()),
            _ => Err(SteamApiInitError::from_raw(result, err_msg)),
        }
    }

    const fn get_all_version() -> [&'static [u8]; 28] {
        [
            sys::STEAMUTILS_INTERFACE_VERSION,
            sys::STEAMNETWORKINGUTILS_INTERFACE_VERSION,
            sys::STEAMAPPLIST_INTERFACE_VERSION,
            sys::STEAMAPPS_INTERFACE_VERSION,
            sys::STEAMCONTROLLER_INTERFACE_VERSION,
            sys::STEAMFRIENDS_INTERFACE_VERSION,
            sys::STEAMGAMESEARCH_INTERFACE_VERSION,
            sys::STEAMHTMLSURFACE_INTERFACE_VERSION,
            sys::STEAMHTTP_INTERFACE_VERSION,
            sys::STEAMINPUT_INTERFACE_VERSION,
            sys::STEAMINVENTORY_INTERFACE_VERSION,
            sys::STEAMMATCHMAKINGSERVERS_INTERFACE_VERSION,
            sys::STEAMMATCHMAKING_INTERFACE_VERSION,
            sys::STEAMMUSICREMOTE_INTERFACE_VERSION,
            sys::STEAMMUSIC_INTERFACE_VERSION,
            sys::STEAMNETWORKINGMESSAGES_INTERFACE_VERSION,
            sys::STEAMNETWORKINGSOCKETS_INTERFACE_VERSION,
            sys::STEAMNETWORKING_INTERFACE_VERSION,
            sys::STEAMPARENTALSETTINGS_INTERFACE_VERSION,
            sys::STEAMPARTIES_INTERFACE_VERSION,
            sys::STEAMREMOTEPLAY_INTERFACE_VERSION,
            sys::STEAMREMOTESTORAGE_INTERFACE_VERSION,
            sys::STEAMSCREENSHOTS_INTERFACE_VERSION,
            sys::STEAMUGC_INTERFACE_VERSION,
            sys::STEAMUSERSTATS_INTERFACE_VERSION,
            sys::STEAMUSER_INTERFACE_VERSION,
            sys::STEAMVIDEO_INTERFACE_VERSION,
            b"\0",
        ]
    }

    unsafe fn proceed_callback(&self, callback: sys::CallbackMsg_t) {
        match callback.m_iCallback {
            sys::SteamShutdown_t_k_iCallback => {
                let value = SteamShutdown::from_raw(SteamShutdown::from_ptr(callback.m_pubParam));
                self.callback_container
                    .steam_shutdown_callback
                    .proceed(value);
            }
            _ => {}
        }
    }

    pub(crate) fn shutdown(&self) {
        unsafe {
            sys::SteamAPI_Shutdown();
        }

        self.callback_container
            .steam_shutdown_callback
            .proceed(SteamShutdown)
    }
}

impl Deref for SteamClientInner {
    type Target = SteamUtils;

    fn deref(&self) -> &Self::Target {
        &self.steam_utils
    }
}
