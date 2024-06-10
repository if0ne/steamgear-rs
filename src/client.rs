use std::{ops::Deref, sync::Arc};

use crate::core::{enums::SteamApiInitError, SteamClientInner};

#[derive(Clone, Debug)]
pub struct SteamClient(Arc<SteamClientInner>);

impl SteamClient {
    pub fn new_client() -> Result<Self, SteamApiInitError> {
        Ok(Self(Arc::new(SteamClientInner::new(false)?)))
    }

    pub fn new_server() -> Result<Self, SteamApiInitError> {
        Ok(Self(Arc::new(SteamClientInner::new(true)?)))
    }

    pub fn shutdown(&self) {
        let count = Arc::strong_count(&self.0);
        if count > 1 {
            tracing::warn!("Called shutdown when amount clones of client is {}", count);
        }

        self.0.shutdown();
    }
}

impl Deref for SteamClient {
    type Target = SteamClientInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
