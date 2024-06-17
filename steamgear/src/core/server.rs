use super::callback::{CallbackDispatcher, CallbackType, CallbackTyped, ClientCallbackContainer};
use super::enums::{ServerMode, SteamApiInitError};
use super::structs::AppId;
use super::{SteamApiInterface, SteamApiState, STEAM_INIT_STATUS};

use crate::utils::callbacks::SteamShutdown;

use steamgear_sys as sys;
use tracing::{error, warn};

#[derive(Debug)]
pub struct SteamApiServer {
    pipe: sys::HSteamPipe,

    //TODO: Replace
    pub(crate) callback_container: ClientCallbackContainer,
}

unsafe impl Send for SteamApiServer {}
unsafe impl Sync for SteamApiServer {}

impl SteamApiServer {
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

                    if let Some((_, sender)) = self.callback_container.call_results.remove(&id) {
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

impl SteamApiServer {
    fn init_internal(
        addr: &std::net::SocketAddrV4,
        query_port: u16,
        mode: ServerMode,
        version: &(u8, u8, u8, u8),
    ) -> Result<(), SteamApiInitError> {
        let versions = Self::get_server_interfaces();
        let versions: Vec<u8> = versions.into_iter().flatten().cloned().collect();
        let versions = versions.as_ptr() as *const ::std::os::raw::c_char;

        let mut err_msg: sys::SteamErrMsg = [0; 1024];

        let ip = u32::from_be_bytes(addr.ip().octets());
        let version = format!("{}.{}.{}.{}", version.0, version.1, version.2, version.3).as_ptr();

        let result = unsafe {
            sys::SteamInternal_GameServer_Init_V2(
                ip,
                addr.port(),
                query_port,
                mode as _,
                version as *const _,
                versions,
                &mut err_msg,
            )
        };

        match result {
            steamgear_sys::ESteamAPIInitResult_k_ESteamAPIInitResult_OK => Ok(()),
            _ => Err(SteamApiInitError::from_raw(result, err_msg)),
        }
    }

    unsafe fn proceed_callback(&self, callback: sys::CallbackMsg_t) {
        let Ok(callback_type): Result<CallbackType, _> = callback
            .m_iCallback
            .try_into()
            .map(|ty: u32| ty.try_into())
            .unwrap()
        else {
            warn!("Got unknown callback type: {}", callback.m_iCallback);
            return;
        };

        let is_client = callback_type.is_for_server();

        match (callback_type, is_client) {
            (CallbackType::SteamShutdown, _) => {
                let value = SteamShutdown::from_raw(SteamShutdown::from_ptr(callback.m_pubParam));
                self.callback_container
                    .steam_shutdown_callback
                    .proceed(value);
            }
            (callback_type, true) => {
                error!(
                    "Bug in steamgear. Didn't handle server callback: {:?}",
                    callback_type
                );
            }
            _ => {}
        }
    }

    const fn get_server_interfaces() -> [&'static [u8]; 11] {
        [
            sys::STEAMUTILS_INTERFACE_VERSION,
            sys::STEAMNETWORKINGUTILS_INTERFACE_VERSION,
            sys::STEAMGAMESERVER_INTERFACE_VERSION,
            sys::STEAMGAMESERVERSTATS_INTERFACE_VERSION,
            sys::STEAMHTTP_INTERFACE_VERSION,
            sys::STEAMINVENTORY_INTERFACE_VERSION,
            sys::STEAMNETWORKING_INTERFACE_VERSION,
            sys::STEAMNETWORKINGMESSAGES_INTERFACE_VERSION,
            sys::STEAMNETWORKINGSOCKETS_INTERFACE_VERSION,
            sys::STEAMUGC_INTERFACE_VERSION,
            b"\0",
        ]
    }
}

impl SteamApiInterface for SteamApiServer {
    type InitArgs = (
        Option<AppId>,
        std::net::SocketAddrV4,
        u16,
        ServerMode,
        (u8, u8, u8, u8),
    );

    fn init(args: Self::InitArgs) -> Result<Self, SteamApiInitError>
    where
        Self: Sized,
    {
        let (app_id, addr, query_port, mode, version) = args;
        unsafe {
            if let Some(app_id) = app_id {
                let app_id = app_id.0.to_string();
                std::env::set_var("SteamAppId", &app_id);
                std::env::set_var("SteamGameId", &app_id);
            }

            let pipe = sys::SteamGameServer_GetHSteamPipe();

            if STEAM_INIT_STATUS
                .compare_exchange(
                    SteamApiState::Stopped as u8,
                    SteamApiState::Init as u8,
                    std::sync::atomic::Ordering::AcqRel,
                    std::sync::atomic::Ordering::SeqCst,
                )
                .is_ok()
            {
                Self::init_internal(&addr, query_port, mode, &version)?;

                sys::SteamAPI_ManualDispatch_Init();
            }

            Ok(Self {
                pipe,
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
                        unsafe {
                            sys::SteamGameServer_Shutdown();
                        }

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
