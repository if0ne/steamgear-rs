pub mod callbacks;
pub mod structs;

use std::ffi::CStr;
use std::sync::Arc;

use chrono::DateTime;
use steamgear_sys as sys;
use structs::{DlcDownloadProgress, DlcInformation, FileDetails, FileNotFound, TrialTime};

use crate::core::{
    callback::CallbackContainer,
    structs::{AppId, DepotId, SteamId},
};

#[derive(Clone, Debug)]
pub struct SteamApps {
    raw: *mut sys::ISteamApps,
    container: Arc<CallbackContainer>,
}

unsafe impl Send for SteamApps {}
unsafe impl Sync for SteamApps {}

impl SteamApps {
    pub(crate) fn new(container: Arc<CallbackContainer>) -> Self {
        unsafe {
            SteamApps {
                raw: sys::SteamAPI_SteamApps_v008(),
                container,
            }
        }
    }
}

impl SteamApps {
    pub fn is_subscribe(&self) -> bool {
        unsafe { sys::SteamAPI_ISteamApps_BIsSubscribed(self.raw) }
    }

    pub fn is_subscribed_from_family_sharing(&self) -> bool {
        unsafe { sys::SteamAPI_ISteamApps_BIsSubscribedFromFamilySharing(self.raw) }
    }

    pub fn is_low_violence(&self) -> bool {
        unsafe { sys::SteamAPI_ISteamApps_BIsLowViolence(self.raw) }
    }

    pub fn is_cybercafe(&self) -> bool {
        unsafe { sys::SteamAPI_ISteamApps_BIsCybercafe(self.raw) }
    }

    pub fn is_vac_banned(&self) -> bool {
        unsafe { sys::SteamAPI_ISteamApps_BIsVACBanned(self.raw) }
    }

    pub fn get_current_game_language(&self) -> String {
        unsafe {
            let raw = sys::SteamAPI_ISteamApps_GetCurrentGameLanguage(self.raw);

            let str = CStr::from_ptr(raw as *mut _).to_string_lossy().to_string();

            str
        }
    }

    pub fn get_available_game_languages(&self) -> Vec<String> {
        unsafe {
            let raw = sys::SteamAPI_ISteamApps_GetAvailableGameLanguages(self.raw);

            let raw = CStr::from_ptr(raw as *mut _);
            let langs = raw.to_str().unwrap();

            langs
                .split(',')
                .into_iter()
                .map(|lang| lang.to_string())
                .collect::<Vec<_>>()
        }
    }

    pub fn is_subscribe_app(&self, app_id: AppId) -> bool {
        unsafe { sys::SteamAPI_ISteamApps_BIsSubscribedApp(self.raw, app_id.0) }
    }

    pub fn is_dlc_installed(&self, dlc_id: AppId) -> bool {
        unsafe { sys::SteamAPI_ISteamApps_BIsDlcInstalled(self.raw, dlc_id.0) }
    }

    pub fn purchase_time(&self, app_id: AppId) -> u32 {
        unsafe { sys::SteamAPI_ISteamApps_GetEarliestPurchaseUnixTime(self.raw, app_id.0) }
    }

    pub fn purchase_date(&self, app_id: AppId) -> DateTime<chrono::Utc> {
        let timestamp =
            unsafe { sys::SteamAPI_ISteamApps_GetEarliestPurchaseUnixTime(self.raw, app_id.0) };

        DateTime::from_timestamp(timestamp as _, 0).expect("invalid timestamp")
    }

    pub fn is_subscribed_from_free_weekend(&self) -> bool {
        unsafe { sys::SteamAPI_ISteamApps_BIsSubscribedFromFreeWeekend(self.raw) }
    }

    pub fn get_dlc_information(&self) -> impl Iterator<Item = DlcInformation> + '_ {
        unsafe {
            let dlc_count = sys::SteamAPI_ISteamApps_GetDLCCount(self.raw);
            (0..dlc_count).filter_map(|i| {
                let mut app_id = 0;
                let mut available = false;
                let mut name_buffer = [0; 128];
                if sys::SteamAPI_ISteamApps_BGetDLCDataByIndex(
                    self.raw,
                    i,
                    &mut app_id,
                    &mut available,
                    name_buffer.as_mut_ptr(),
                    128,
                ) {
                    let dlc_name = CStr::from_ptr(name_buffer.as_mut_ptr())
                        .to_string_lossy()
                        .to_string();
                    Some(DlcInformation {
                        dlc_id: AppId(app_id),
                        dlc_name,
                        available,
                    })
                } else {
                    None
                }
            })
        }
    }

    pub fn uninstall_dlc(&self, dlc_id: AppId) {
        unsafe {
            sys::SteamAPI_ISteamApps_UninstallDLC(self.raw, dlc_id.0);
        }
    }

    pub fn get_current_beta_name(&self) -> Option<String> {
        unsafe {
            let mut name_buffer = [0; 128];

            if sys::SteamAPI_ISteamApps_GetCurrentBetaName(self.raw, name_buffer.as_mut_ptr(), 128)
            {
                Some(
                    CStr::from_ptr(name_buffer.as_mut_ptr())
                        .to_string_lossy()
                        .to_string(),
                )
            } else {
                None
            }
        }
    }

    pub fn mark_content_corrupt(&self, missing_files_only: bool) {
        unsafe {
            sys::SteamAPI_ISteamApps_MarkContentCorrupt(self.raw, missing_files_only);
        }
    }

    pub fn get_installed_depots(&self, app_id: AppId) -> impl Iterator<Item = DepotId> + 'static {
        unsafe {
            let mut depots = [0u32; 32];
            let count = sys::SteamAPI_ISteamApps_GetInstalledDepots(
                self.raw,
                app_id.0,
                depots.as_mut_ptr(),
                32,
            ) as usize;

            depots.into_iter().take(count).map(|id| DepotId(id))
        }
    }

    pub fn get_app_install_dir(&self, app_id: AppId) -> Option<std::path::PathBuf> {
        unsafe {
            let mut name_buffer = [0; 128];

            if sys::SteamAPI_ISteamApps_GetAppInstallDir(
                self.raw,
                app_id.0,
                name_buffer.as_mut_ptr(),
                128,
            ) > 0
            {
                Some(std::path::PathBuf::from(
                    CStr::from_ptr(name_buffer.as_mut_ptr())
                        .to_string_lossy()
                        .to_string(),
                ))
            } else {
                None
            }
        }
    }

    pub fn is_app_installed(&self, app_id: AppId) -> bool {
        unsafe { sys::SteamAPI_ISteamApps_BIsAppInstalled(self.raw, app_id.0) }
    }

    pub fn get_app_owner(&self) -> SteamId {
        unsafe { SteamId(sys::SteamAPI_ISteamApps_GetAppOwner(self.raw)) }
    }

    pub fn get_launch_query_param(&self, key: impl AsRef<std::ffi::CStr>) -> String {
        unsafe {
            let key = key.as_ref();

            let result = sys::SteamAPI_ISteamApps_GetLaunchQueryParam(self.raw, key.as_ptr());

            CStr::from_ptr(result as *mut _)
                .to_string_lossy()
                .to_string()
        }
    }

    pub fn get_dlc_download_progress(&self, dlc_id: AppId) -> Option<DlcDownloadProgress> {
        unsafe {
            let mut downloaded = 0;
            let mut total = 0;
            if sys::SteamAPI_ISteamApps_GetDlcDownloadProgress(
                self.raw,
                dlc_id.0,
                &mut downloaded,
                &mut total,
            ) {
                Some(DlcDownloadProgress { downloaded, total })
            } else {
                None
            }
        }
    }

    pub fn get_app_build_id(&self) -> i32 {
        unsafe { sys::SteamAPI_ISteamApps_GetAppBuildId(self.raw) }
    }

    pub async fn get_file_details(
        &self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<FileDetails, FileNotFound> {
        let path = path.as_ref();
        let path = path.as_os_str().as_encoded_bytes().as_ptr();

        let call_id =
            unsafe { sys::SteamAPI_ISteamApps_GetFileDetails(self.raw, path as *const _) };

        self.container
            .register_call_result::<FileDetails>(call_id)
            .await
    }

    pub fn get_launch_command_line(&self) -> String {
        unsafe {
            let mut name_buffer = [0; 128];

            CStr::from_ptr(name_buffer.as_ptr() as *mut _)
                .to_string_lossy()
                .to_string()
        }
    }

    pub fn is_timed_trial(&self) -> Option<TrialTime> {
        unsafe {
            let mut allowed = 0;
            let mut played = 0;
            if sys::SteamAPI_ISteamApps_BIsTimedTrial(self.raw, &mut allowed, &mut played) {
                Some(TrialTime { allowed, played })
            } else {
                None
            }
        }
    }
}
