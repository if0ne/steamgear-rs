use steamgear_sys as sys;

#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum SteamApiInitError {
    #[error("{0}")]
    FailedGeneric(String),
    #[error("{0}")]
    NoSteamClient(String),
    #[error("{0}")]
    VersionMismatch(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum ServerMode {
    Invalid = sys::EServerMode_eServerModeInvalid as u32,
    NoAuthentication = sys::EServerMode_eServerModeNoAuthentication as u32,
    Authentication = sys::EServerMode_eServerModeAuthentication as u32,
    AuthenticationAndSecure = sys::EServerMode_eServerModeAuthenticationAndSecure as u32,
}
