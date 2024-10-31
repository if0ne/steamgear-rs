use steamgear_sys as sys;

use crate::{
    internal::{
        core::{reactor::REACTOR, SteamApiState, STEAM_INIT_STATUS},
        sealed::Sealed,
    },
    SteamApiInterface,
};

use super::{
    enums::{ServerMode, SteamApiInitError},
    structs::AppId,
};

pub struct Server {
    pipe: sys::HSteamPipe,
}

impl Server {
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

                    REACTOR.wake(id);
                } else {
                    //self.proceed_callback(callback);
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

impl Server {
    pub(crate) fn init(
        app_id: Option<AppId>,
        addr: std::net::SocketAddrV4,
        query_port: u16,
        mode: ServerMode,
        version: (u8, u8, u8, u8),
    ) -> Result<Self, SteamApiInitError>
    where
        Self: Sized,
    {
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

            Ok(Self { pipe })
        }
    }

    pub(crate) fn shutdown(&self) {
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
    }

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

impl SteamApiInterface for Server {}
impl Sealed for Server {}
