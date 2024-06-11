use std::{fmt::Display, ops::Deref};

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct SteamId(pub(crate) u64);

impl Deref for SteamId {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<u64> for SteamId {
    fn as_ref(&self) -> &u64 {
        &self.0
    }
}

impl Display for SteamId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SteamId({})", self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct GameId(pub(crate) u64);

impl Deref for GameId {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<u64> for GameId {
    fn as_ref(&self) -> &u64 {
        &self.0
    }
}

impl Display for GameId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GameId({})", self.0)
    }
}
