use super::callback::{CallbackContainer, CallbackDispatcher, CallbackTyped};
use super::enums::SteamApiInitError;
use super::{AppId, SteamApiInterface, SteamApiState, STEAM_INIT_STATUS};

use crate::apps::SteamApps;
use crate::utils::callbacks::SteamShutdown;
use crate::utils::client::SteamUtilsClient;

use dashmap::DashMap;
use futures::channel::oneshot::Sender;
use steamgear_sys as sys;

#[derive(Debug)]
pub struct SteamApiClient {
    pipe: sys::HSteamPipe,
    call_results: DashMap<sys::SteamAPICall_t, Sender<sys::CallbackMsg_t>>,

    pub(crate) callback_container: CallbackContainer,
    pub(crate) steam_utils: SteamUtilsClient,
    pub(crate) steam_apps: SteamApps,
}

unsafe impl Send for SteamApiClient {}
unsafe impl Sync for SteamApiClient {}

impl SteamApiInterface for SteamApiClient {
    type InitArgs = (Option<AppId>,);

    fn init(args: Self::InitArgs) -> Result<Self, SteamApiInitError>
    where
        Self: Sized,
    {
        let (app_id,) = args;
        unsafe {
            if let Some(app_id) = app_id {
                let app_id = app_id.to_string();
                std::env::set_var("SteamAppId", &app_id);
                std::env::set_var("SteamGameId", &app_id);
            }

            let pipe = sys::SteamAPI_GetHSteamPipe();

            if STEAM_INIT_STATUS
                .compare_exchange(
                    SteamApiState::Stopped as u8,
                    SteamApiState::Init as u8,
                    std::sync::atomic::Ordering::AcqRel,
                    std::sync::atomic::Ordering::SeqCst,
                )
                .is_ok()
            {
                Self::init_internal()?;

                sys::SteamAPI_ManualDispatch_Init();
            }

            Ok(Self {
                pipe,
                call_results: Default::default(),
                steam_utils: SteamUtilsClient::new(),
                steam_apps: SteamApps::new(),
                callback_container: Default::default(),
            })
        }
    }

    fn shutdown(&self) {
        loop {
            let current = STEAM_INIT_STATUS.load(std::sync::atomic::Ordering::Acquire);

            if current != SteamApiState::Stopped as u8 {
                match STEAM_INIT_STATUS.compare_exchange(
                    current,
                    SteamApiState::Stopped as u8,
                    std::sync::atomic::Ordering::SeqCst,
                    std::sync::atomic::Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        unsafe { sys::SteamAPI_Shutdown() }

                        break;
                    }
                    Err(_) => continue,
                }
            } else {
                break;
            }
        }

        self.callback_container
            .steam_shutdown_callback
            .proceed(SteamShutdown)
    }
}

impl SteamApiClient {
    pub fn restart_app_if_necessary(&self, app_id: u32) -> bool {
        unsafe { sys::SteamAPI_RestartAppIfNecessary(app_id) }
    }

    pub fn release_current_thread_memory(&self) {
        unsafe {
            sys::SteamAPI_ReleaseCurrentThreadMemory();
        }
    }

    pub fn run_callbacks(&self) {
        unsafe {
            sys::SteamAPI_ManualDispatch_RunFrame(self.pipe);
            let mut callback = std::mem::zeroed();

            if STEAM_INIT_STATUS
                .compare_exchange(
                    SteamApiState::Init as u8,
                    SteamApiState::RunCallbacks as u8,
                    std::sync::atomic::Ordering::SeqCst,
                    std::sync::atomic::Ordering::Relaxed,
                )
                .is_err()
            {
                return;
            }

            while sys::SteamAPI_ManualDispatch_GetNextCallback(self.pipe, &mut callback) {
                if callback.m_iCallback as u32 == sys::SteamAPICallCompleted_t_k_iCallback as u32 {
                    let apicall =
                        &*(callback.m_pubParam as *const _ as *const sys::SteamAPICallCompleted_t);
                    let id = apicall.m_hAsyncCall;

                    if let Some((_, sender)) = self.call_results.remove(&id) {
                        match sender.send(callback) {
                            Ok(_) => {
                                tracing::debug!("Sent call result with id: {}", id)
                            }
                            Err(_) => {
                                tracing::debug!(
                                    "CallResult with id {} have received, but receiver is broken",
                                    id
                                )
                            }
                        }
                    }
                } else {
                    self.proceed_callback(callback);
                }
                sys::SteamAPI_ManualDispatch_FreeLastCallback(self.pipe);
            }

            let _ = STEAM_INIT_STATUS.compare_exchange(
                SteamApiState::RunCallbacks as u8,
                SteamApiState::Init as u8,
                std::sync::atomic::Ordering::SeqCst,
                std::sync::atomic::Ordering::Relaxed,
            );
        }
    }
}

impl SteamApiClient {
    pub(crate) async fn register_call_result<T: CallbackTyped>(
        &self,
        id: sys::SteamAPICall_t,
    ) -> T {
        let (sender, receiver) = futures::channel::oneshot::channel();
        self.call_results.insert(id, sender);
        let result = receiver.await.expect("Client dropped");

        assert_eq!(std::mem::size_of::<T::Raw>(), result.m_cubParam as usize);

        let raw_data = unsafe { T::from_ptr(result.m_pubParam) };
        T::from_raw(raw_data)
    }
}

impl SteamApiClient {
    fn init_internal() -> Result<(), SteamApiInitError> {
        let versions = Self::get_client_interfaces();
        let versions: Vec<u8> = versions.into_iter().flatten().cloned().collect();
        let versions = versions.as_ptr() as *const ::std::os::raw::c_char;

        let mut err_msg: sys::SteamErrMsg = [0; 1024];

        let result = unsafe { sys::SteamInternal_SteamAPI_Init(versions, &mut err_msg) };

        match result {
            steamgear_sys::ESteamAPIInitResult_k_ESteamAPIInitResult_OK => Ok(()),
            _ => Err(SteamApiInitError::from_raw(result, err_msg)),
        }
    }

    const fn get_client_interfaces() -> [&'static [u8]; 27] {
        [
            sys::STEAMUTILS_INTERFACE_VERSION,
            sys::STEAMNETWORKINGUTILS_INTERFACE_VERSION,
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
        match callback.m_iCallback.try_into().unwrap() {
            sys::SteamShutdown_t_k_iCallback => {
                let value = SteamShutdown::from_raw(SteamShutdown::from_ptr(callback.m_pubParam));
                self.callback_container
                    .steam_shutdown_callback
                    .proceed(value);
            }
            _ => {}
        }
    }
}
