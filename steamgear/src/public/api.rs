use std::{ops::Deref, sync::Arc};

use crate::internal::sealed::Sealed;

use super::{
    client::Client,
    enums::{ServerMode, SteamApiInitError},
    server::Server,
    structs::AppId,
};

#[derive(Clone, Debug)]
pub struct SteamApi<T: SteamApiInterface>(Arc<T>);

pub trait SteamApiInterface: Send + Sync + Sealed {}

impl SteamApi<Client> {
    pub fn new_client(app_id: Option<AppId>) -> Result<Self, SteamApiInitError> {
        let client = Client::init(app_id)?;
        Ok(Self(Arc::new(client)))
    }

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

impl SteamApi<Server> {
    pub fn new_server(
        app_id: Option<AppId>,
        addr: std::net::SocketAddrV4,
        query_port: u16,
        mode: ServerMode,
        version: (u8, u8, u8, u8),
    ) -> Result<Self, SteamApiInitError> {
        let server = Server::init(app_id, addr, query_port, mode, version)?;
        Ok(Self(Arc::new(server)))
    }

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

impl<T: SteamApiInterface> Deref for SteamApi<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
