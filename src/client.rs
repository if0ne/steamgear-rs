use std::{ops::Deref, sync::Arc, task::Waker};

use dashmap::DashMap;

use steamgear_sys as sys;

use crate::utils::SteamUtils;

#[derive(Clone, Debug)]
pub struct SteamClient(Arc<SteamClientInner>);

impl SteamClient {
    pub fn new_client() -> Result<Self, String> {
        Ok(Self(Arc::new(SteamClientInner::new(false)?)))
    }

    pub fn new_server() -> Result<Self, String> {
        Ok(Self(Arc::new(SteamClientInner::new(true)?)))
    }
}

impl Deref for SteamClient {
    type Target = SteamClientInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct SteamClientInner {
    pipe: sys::HSteamPipe,
    call_results: DashMap<sys::SteamAPICall_t, Waker>,

    pub(crate) steam_utils: SteamUtils,
}

impl SteamClientInner {
    pub fn run_callbacks(&self) {
        unsafe {
            sys::SteamAPI_ManualDispatch_RunFrame(self.pipe);
            let mut callback = std::mem::zeroed();
            while sys::SteamAPI_ManualDispatch_GetNextCallback(self.pipe, &mut callback) {
                if callback.m_iCallback == sys::SteamAPICallCompleted_t_k_iCallback as i32 {
                    let apicall =
                        &*(callback.m_pubParam as *const _ as *const sys::SteamAPICallCompleted_t);
                    let id = apicall.m_hAsyncCall;

                    if let Some(entry) = self.call_results.get(&id) {
                        entry.value().wake_by_ref();
                    }
                } else {
                    // TODO: Callback call
                }
                sys::SteamAPI_ManualDispatch_FreeLastCallback(self.pipe);
            }
        }
    }
}

impl SteamClientInner {
    fn init_ex() -> Result<(), String> {
        let versions: Vec<&[u8]> = vec![
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
        ];

        let versions: Vec<u8> = versions.into_iter().flatten().cloned().collect();
        let versions = versions.as_ptr() as *const ::std::os::raw::c_char;

        let mut err_msg: sys::SteamErrMsg = [0; 1024];

        let result = unsafe { sys::SteamInternal_SteamAPI_Init(versions, &mut err_msg) };

        let err_string = unsafe {
            let cstr = std::ffi::CStr::from_ptr(err_msg.as_ptr() as *const std::ffi::c_char);
            cstr.to_string_lossy().to_owned().into_owned()
        };

        match result {
            steamgear_sys::ESteamAPIInitResult::k_ESteamAPIInitResult_OK => Ok(()),
            _ => Err(err_string),
        }
    }

    pub(crate) fn new(is_server: bool) -> Result<Self, String> {
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
            })
        }
    }

    pub(crate) fn register_call_result(&self, id: sys::SteamAPICall_t, waker: Waker) {
        self.call_results.insert(id, waker);
    }

    pub(crate) fn remove_call_result(&self, id: sys::SteamAPICall_t) {
        self.call_results.remove(&id);
    }
}

impl Deref for SteamClientInner {
    type Target = SteamUtils;

    fn deref(&self) -> &Self::Target {
        &self.steam_utils
    }
}
