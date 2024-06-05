use std::{ops::Deref, sync::Arc, task::Waker};

use dashmap::DashMap;

use steamgear_sys::{
    HSteamPipe, SteamAPICallCompleted_t, SteamAPICallCompleted_t_k_iCallback, SteamAPICall_t,
    SteamAPI_GetHSteamPipe, SteamAPI_ManualDispatch_FreeLastCallback,
    SteamAPI_ManualDispatch_GetNextCallback, SteamAPI_ManualDispatch_Init,
    SteamAPI_ManualDispatch_RunFrame, SteamGameServer_GetHSteamPipe,
};

use crate::utils::SteamUtils;

#[derive(Clone, Debug)]
pub struct SteamClient(Arc<SteamClientInner>);

impl SteamClient {
    pub fn new_client() -> Self {
        Self(Arc::new(SteamClientInner::new(false)))
    }

    pub fn new_server() -> Self {
        Self(Arc::new(SteamClientInner::new(true)))
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
    pipe: HSteamPipe,
    call_results: DashMap<SteamAPICall_t, Waker>,

    pub(crate) steam_utils: SteamUtils,
}

impl SteamClientInner {
    pub fn run_callbacks(&self) {
        unsafe {
            SteamAPI_ManualDispatch_RunFrame(self.pipe);
            let mut callback = std::mem::zeroed();
            while SteamAPI_ManualDispatch_GetNextCallback(self.pipe, &mut callback) {
                if callback.m_iCallback == SteamAPICallCompleted_t_k_iCallback as i32 {
                    let apicall =
                        &*(callback.m_pubParam as *const _ as *const SteamAPICallCompleted_t);
                    let id = apicall.m_hAsyncCall;

                    if let Some(entry) = self.call_results.get(&id) {
                        entry.value().wake_by_ref();
                    }
                } else {
                    // TODO: Callback call
                }
                SteamAPI_ManualDispatch_FreeLastCallback(self.pipe);
            }
        }
    }
}

impl SteamClientInner {
    pub(crate) fn new(is_server: bool) -> Self {
        unsafe {
            let pipe = if is_server {
                SteamGameServer_GetHSteamPipe()
            } else {
                SteamAPI_GetHSteamPipe()
            };

            SteamAPI_ManualDispatch_Init();

            Self {
                pipe,
                call_results: Default::default(),
                steam_utils: SteamUtils::new(),
            }
        }
    }

    pub(crate) fn register_call_result(&self, id: SteamAPICall_t, waker: Waker) {
        self.call_results.insert(id, waker);
    }

    pub(crate) fn remove_call_result(&self, id: SteamAPICall_t) {
        self.call_results.remove(&id);
    }
}

impl Deref for SteamClientInner {
    type Target = SteamUtils;

    fn deref(&self) -> &Self::Target {
        &self.steam_utils
    }
}
