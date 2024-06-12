use crate::core::{
    callback::{CallbackDispatcher, CallbackError, CallbackTyped},
    client::SteamApiClient,
    AppId,
};

use steamgear_sys as sys;

#[derive(Clone, Copy, Debug)]
pub struct DlcInstalled {
    pub id: AppId,
}

impl CallbackTyped for DlcInstalled {
    const TYPE: u32 = sys::DlcInstalled_t_k_iCallback as u32;
    type Raw = sys::DlcInstalled_t;

    fn from_raw(raw: Self::Raw) -> Self {
        DlcInstalled { id: raw.m_nAppID }
    }
}

impl SteamApiClient {
    pub async fn install_app(&self, app_id: AppId) -> Result<DlcInstalled, CallbackError> {
        let recv = self.callback_container
            .dlc_installed_callback
            .register();

        unsafe {
            sys::SteamAPI_ISteamApps_InstallDLC(self.steam_apps.0, app_id);
        }

        recv
            .await
            .map_err(|_| CallbackError::Canceled)
    }
}
