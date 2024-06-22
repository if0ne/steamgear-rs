#![allow(private_bounds)]

use std::{ops::Deref, sync::Arc};

use crate::core::{
    client::SteamApiClient,
    enums::{ServerMode, SteamApiInitError},
    server::SteamApiServer,
    structs::AppId,
    SteamApiInterface,
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
    pub fn new_client(app_id: Option<AppId>) -> Result<Self, SteamApiInitError> {
        let client = SteamApiClient::init((app_id,))?;
        Ok(Self(Arc::new(client)))
    }
}

impl SteamApi<SteamApiServer> {
    pub fn new_server(
        app_id: Option<AppId>,
        addr: std::net::SocketAddrV4,
        query_port: u16,
        mode: ServerMode,
        version: (u8, u8, u8, u8),
    ) -> Result<Self, SteamApiInitError> {
        let server = SteamApiServer::init((app_id, addr, query_port, mode, version))?;
        Ok(Self(Arc::new(server)))
    }
}

impl<T: SteamApiInterface> Deref for SteamApi<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
