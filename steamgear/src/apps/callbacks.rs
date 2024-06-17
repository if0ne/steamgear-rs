use crate::core::{
    callback::{CallbackDispatcher, CallbackType, CallbackTyped},
    structs::AppId,
};

use futures::{Stream, StreamExt};
use steamgear_sys as sys;

use super::SteamApps;

#[derive(Clone, Copy, Debug)]
pub struct DlcInstalled {
    pub id: AppId,
}

impl CallbackTyped for DlcInstalled {
    const TYPE: CallbackType = CallbackType::DlcInstalled;
    type Raw = sys::DlcInstalled_t;
    type Mapped = Self;

    fn from_raw(raw: Self::Raw) -> Self::Mapped {
        DlcInstalled {
            id: AppId(raw.m_nAppID),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct NewUrlLaunchParams;

impl CallbackTyped for NewUrlLaunchParams {
    const TYPE: CallbackType = CallbackType::NewUrlLaunchParameters;
    type Raw = sys::NewUrlLaunchParameters_t;
    type Mapped = Self;

    fn from_raw(_: Self::Raw) -> Self::Mapped {
        NewUrlLaunchParams
    }
}

impl SteamApps {
    pub async fn install_dlc(&self, app_id: AppId) -> DlcInstalled {
        let recv = &mut *self.container.dlc_installed_callback.register();

        unsafe {
            sys::SteamAPI_ISteamApps_InstallDLC(self.raw, app_id.0);
        }

        recv.next().await.unwrap()
    }

    pub fn on_new_launch_query_param(&self) -> impl Stream<Item = NewUrlLaunchParams> {
        self.container.new_url_launch_params_callback.register()
    }
}
