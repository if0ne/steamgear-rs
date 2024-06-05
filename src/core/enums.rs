#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum SteamApiInitError {
    FailedGeneric(String),
    NoSteamClient(String),
    VersionMismatch(String),
}
