use std::{ops::Deref, sync::Arc};

use crate::{
    core::enums::SteamApiInitError,
    prelude::{
        client::SteamApiClient, enums::ServerMode, server::SteamApiServer, SteamApiInterface,
    },
};

#[derive(Clone, Debug)]
pub struct SteamApi<T: SteamApiInterface>(Arc<T>);

impl<T: SteamApiInterface> SteamApi<T> {
    pub fn shutdown(&self) {
        let count = Arc::strong_count(&self.0);
        if count > 1 {
            tracing::warn!(
                "Called shutdown when amount clones of steam api is {}",
                count
            );
        }

        self.0.shutdown();
    }
}

impl SteamApi<SteamApiClient> {
    pub fn new_client() -> Result<Self, SteamApiInitError> {
        let client = SteamApiClient::init(())?;
        Ok(Self(Arc::new(client)))
    }
}

impl SteamApi<SteamApiServer> {
    pub fn new_server(
        addr: std::net::SocketAddrV4,
        query_port: u16,
        mode: ServerMode,
        version: (u8, u8, u8, u8),
    ) -> Result<Self, SteamApiInitError> {
        let server = SteamApiServer::init((addr, query_port, mode, version))?;
        Ok(Self(Arc::new(server)))
    }
}

impl<T: SteamApiInterface> Deref for SteamApi<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
